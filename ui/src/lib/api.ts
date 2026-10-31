import { invoke } from "@tauri-apps/api/core";

export interface ServerSummary {
  link: string;
  name: string;
  version: string;
  loader: string;
  loaderVersion: string | null;
  fileCount: number;
  versionHash: string;
  commit: string;
  updates: string[];
}

export interface AccountSummary {
  kind: "syncraft" | "offline";
  name: string;
  email: string | null;
  uuid: string;
}

export interface JavaSummary {
  path: string;
  version: number;
  selected: boolean;
}

export interface SkinData {
  dataUrl: string;
  avatarUrl: string;
  model: "default" | "slim";
  width: number;
  height: number;
}

export interface Snapshot {
  server: ServerSummary | null;
  link: string | null;
  serverError: string | null;
  account: AccountSummary | null;
  gameDir: string;
  java: JavaSummary | null;
  detectedJava: JavaSummary[];
  source: "default" | "bmclapi";
  concurrency: number | null;
  minimizeOnLaunch: boolean;
  autoLaunch: boolean;
  version: string;
}

export type Phase =
  | "idle"
  | "checking"
  | "installing"
  | "syncing"
  | "launching"
  | "running"
  | "error";

export interface LaunchEvent {
  phase: Phase;
  progress: number;
  stage: string;
  filesDone: number;
  filesTotal: number;
  bytesDone: number;
  bytesTotal: number;
  message?: string;
}

export const api = {
  snapshot: () => invoke<Snapshot>("snapshot"),
  refreshServer: () => invoke<ServerSummary>("refresh_server"),
  setServerLink: (link: string) => invoke<ServerSummary>("set_server_link", { link }),
  login: (email: string, password: string) =>
    invoke<AccountSummary>("login", { email, password }),
  register: (username: string, email: string, password: string) =>
    invoke<{ id: string }>("register", { username, email, password }),
  setOfflineAccount: (username: string) =>
    invoke<AccountSummary>("set_offline_account", { username }),
  logout: () => invoke<void>("logout"),
  detectJava: () => invoke<JavaSummary[]>("detect_java"),
  setJava: (path: string | null) => invoke<void>("set_java", { path }),
  setGameDir: (path: string) => invoke<string>("set_game_dir", { path }),
  setSource: (source: "default" | "bmclapi") =>
    invoke<void>("set_source", { source }),
  setConcurrency: (value: number | null) =>
    invoke<void>("set_concurrency", { value }),
  setToggle: (name: "minimizeOnLaunch" | "autoLaunch", value: boolean) =>
    invoke<void>("set_toggle", { name, value }),
  startLaunch: () => invoke<void>("start_launch"),
  fetchSkin: () => invoke<SkinData | null>("fetch_skin"),
  uploadSkin: (data: string, model: "default" | "slim" | null) =>
    invoke<void>("upload_skin", { data, model }),
  deleteSkin: () => invoke<void>("delete_skin"),
};

