use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use syncraft_core::client::{
    auth,
    config::{Account, client_config},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinData {
    /// Full skin texture as a data URL, for skinview3d.
    pub data_url: String,
    /// Rendered face avatar as a data URL.
    pub avatar_url: String,
    pub model: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
struct Profile {
    #[serde(default)]
    properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
struct Property {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct TexturePayload {
    textures: Textures,
}

#[derive(Debug, Deserialize)]
struct Textures {
    #[serde(rename = "SKIN")]
    skin: Option<SkinTexture>,
}

#[derive(Debug, Deserialize)]
struct SkinTexture {
    url: String,
    #[serde(default)]
    metadata: Option<SkinMetadata>,
}

#[derive(Debug, Deserialize)]
struct SkinMetadata {
    model: Option<String>,
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(25))
        .build()
        .map_err(|e| e.to_string())
}

fn link() -> Result<String, String> {
    client_config()
        .get()
        .syncraft_link
        .filter(|link| !link.is_empty())
        .ok_or_else(|| "请先连接服务器".to_string())
}

fn syncraft_account() -> Result<Account, String> {
    match client_config().get().account {
        Some(account @ Account::Syncraft { .. }) => Ok(account),
        Some(Account::Offline { .. }) => Err("离线账号不支持皮肤".into()),
        None => Err("请先添加账号".into()),
    }
}

/// Refresh the Yggdrasil token and persist it, so skin management keeps working
/// after the access token expires.
async fn authorized_account(link: &str) -> Result<Account, String> {
    let account = syncraft_account()?;
    let Account::Syncraft {
        username,
        uuid,
        email,
        client_token,
        access_token,
    } = account
    else {
        return Err("离线账号不支持皮肤".into());
    };

    let (access_token, client_token) = auth::refresh_token(link, &access_token, &client_token)
        .await
        .map_err(|err| format!("登录已失效：{err}"))?;

    let account = Account::Syncraft {
        username,
        uuid,
        email,
        client_token,
        access_token,
    };
    client_config()
        .update(|cfg| cfg.account = Some(account.clone()))
        .map_err(|e| e.to_string())?;
    Ok(account)
}

/// Rewrite the server's own texture URL (which uses the public `base_url`) to
/// the configured address. External Mojang/LittleSkin URLs are kept as-is.
fn resolve_skin_url(link: &str, url: &str) -> String {
    if let Some(index) = url.find("/yggdrasil/textures/") {
        format!("{link}{}", &url[index..])
    } else {
        url.to_string()
    }
}

#[tauri::command]
pub async fn fetch_skin() -> Result<Option<SkinData>, String> {
    let link = link()?;
    let account = syncraft_account()?;
    let Account::Syncraft { uuid, .. } = account else {
        return Ok(None);
    };

    let http = client()?;
    let profile_url = format!("{link}/yggdrasil/sessionserver/session/minecraft/profile/{uuid}");
    let response = http
        .get(&profile_url)
        .send()
        .await
        .map_err(|e| format!("获取皮肤失败：{e}"))?;
    if !response.status().is_success() {
        return Err(format!("获取皮肤失败：HTTP {}", response.status()));
    }
    let profile: Profile = response
        .json()
        .await
        .map_err(|e| format!("解析皮肤信息失败：{e}"))?;

    let Some(texture) = profile
        .properties
        .iter()
        .find(|property| property.name == "textures")
    else {
        return Ok(None);
    };

    let decoded = STANDARD
        .decode(texture.value.as_bytes())
        .map_err(|e| format!("解析皮肤数据失败：{e}"))?;
    let payload: TexturePayload =
        serde_json::from_slice(&decoded).map_err(|e| format!("解析皮肤数据失败：{e}"))?;
    let Some(skin) = payload.textures.skin else {
        return Ok(None);
    };

    let url = resolve_skin_url(&link, &skin.url);
    let response = http
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载皮肤失败：{e}"))?;
    if !response.status().is_success() {
        return Err(format!("下载皮肤失败：HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("下载皮肤失败：{e}"))?
        .to_vec();

    let (width, height) = syncraft_skin::dimensions(&bytes).map_err(|e| e.to_string())?;
    let avatar = syncraft_skin::render_avatar(&bytes, 256).map_err(|e| e.to_string())?;
    let declared = skin
        .metadata
        .and_then(|metadata| metadata.model)
        .filter(|model| model == "slim" || model == "default");
    let model = match declared {
        Some(model) => model,
        None => {
            if syncraft_skin::detect_slim(&bytes).unwrap_or(false) {
                "slim".to_string()
            } else {
                "default".to_string()
            }
        }
    };

    Ok(Some(SkinData {
        data_url: format!("data:image/png;base64,{}", STANDARD.encode(&bytes)),
        avatar_url: format!("data:image/png;base64,{}", STANDARD.encode(&avatar)),
        model,
        width,
        height,
    }))
}

#[tauri::command]
pub async fn upload_skin(data: String, model: Option<String>) -> Result<(), String> {
    let link = link()?;
    let account = authorized_account(&link).await?;
    let Account::Syncraft {
        uuid, access_token, ..
    } = account
    else {
        return Err("离线账号不支持皮肤".into());
    };

    let bytes = STANDARD
        .decode(data.as_bytes())
        .map_err(|e| format!("无效的皮肤数据：{e}"))?;
    syncraft_skin::dimensions(&bytes).map_err(|e| e.to_string())?;

    let model = match model.as_deref() {
        Some("slim") => "slim".to_string(),
        Some("default") => "default".to_string(),
        _ => {
            if syncraft_skin::detect_slim(&bytes).unwrap_or(false) {
                "slim".to_string()
            } else {
                "default".to_string()
            }
        }
    };
    let boundary = format!(
        "----syncraft{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let mut body = Vec::with_capacity(bytes.len() + 512);
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\n{model}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"skin.png\"\r\nContent-Type: image/png\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(&bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let response = client()?
        .put(format!("{link}/yggdrasil/api/user/profile/{uuid}/skin"))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("Authorization", format!("Bearer {access_token}"))
        .body(body)
        .send()
        .await
        .map_err(|e| format!("上传皮肤失败：{e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "上传皮肤失败：HTTP {} {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_skin() -> Result<(), String> {
    let link = link()?;
    let account = authorized_account(&link).await?;
    let Account::Syncraft {
        uuid, access_token, ..
    } = account
    else {
        return Err("离线账号不支持皮肤".into());
    };

    let response = client()?
        .delete(format!("{link}/yggdrasil/api/user/profile/{uuid}/skin"))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("删除皮肤失败：{e}"))?;

    if !response.status().is_success() {
        return Err(format!("删除皮肤失败：HTTP {}", response.status()));
    }
    Ok(())
}
