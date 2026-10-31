<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { SkinViewer as SkinViewerType } from "skinview3d";

  let {
    skin,
    model,
  }: {
    skin: string;
    model: "default" | "slim";
  } = $props();

  // The top bar capsule transition is 320ms. Start just after it so the two
  // never compete, while keeping the wait short.
  const START_DELAY = 330;

  let container: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  let viewer = $state<SkinViewerType | null>(null);
  let ready = $state(false);
  let shown = false;
  let cancelled = false;
  let timer = 0;
  let observer: ResizeObserver | null = null;

  onMount(() => {
    cancelled = false;

    const start = async () => {
      const { SkinViewer, IdleAnimation } = await import("skinview3d");
      if (cancelled || !container) return;

      const instance = new SkinViewer({
        canvas,
        width: container.clientWidth,
        height: container.clientHeight,
        fov: 35,
        zoom: 0.85,
      });
      instance.controls.enablePan = false;
      instance.controls.enableRotate = true;
      instance.controls.enableZoom = true;
      instance.animation = new IdleAnimation();

      observer = new ResizeObserver(() => {
        if (container) {
          instance.setSize(container.clientWidth, container.clientHeight);
        }
      });
      observer.observe(container);
      viewer = instance;
    };

    timer = window.setTimeout(() => {
      void start();
    }, START_DELAY);
  });

  // Reload whenever the texture or the model changes so a fresh upload and the
  // 经典/苗条 switch show up immediately. The canvas fades in once the first
  // texture is ready.
  $effect(() => {
    const current = viewer;
    if (!current) return;
    const texture = skin;
    const currentModel = model;
    void current
      .loadSkin(texture, { model: currentModel })
      .then(() => {
        if (cancelled || viewer !== current) return;
        if (!shown) {
          shown = true;
          ready = true;
        }
      })
      .catch(() => {});
  });

  onDestroy(() => {
    cancelled = true;
    window.clearTimeout(timer);
    observer?.disconnect();
    // Dispose off the navigation critical path; freeing the WebGL context is
    // not free and would otherwise stutter the next page transition.
    const instance = viewer;
    viewer = null;
    if (instance) {
      const withIdle = window as unknown as {
        requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number;
      };
      if (typeof withIdle.requestIdleCallback === "function") {
        withIdle.requestIdleCallback(() => instance.dispose(), { timeout: 800 });
      } else {
        window.setTimeout(() => instance.dispose(), 0);
      }
    }
  });
</script>

<div class="stage" class:ready bind:this={container}>
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .stage {
    position: relative;
    width: 100%;
    aspect-ratio: 3 / 4;
    overflow: hidden;
    border: 1px solid var(--rule);
    background: var(--paper);
    cursor: grab;
  }

  .stage:active {
    cursor: grabbing;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
    opacity: 0;
    transition: opacity 200ms var(--ease);
  }

  .stage.ready canvas {
    opacity: 1;
  }
</style>
