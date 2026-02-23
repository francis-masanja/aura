<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import GestureController from '$lib/components/GestureController.svelte';
  import { onMount } from 'svelte';

  let activeGesture = $state("");
  let gestureTimer: any;

  async function mediaAction(action: string) {
    showIndicator(action);
    await invoke("media_control", { action });
  }

  async function volumeAction(delta: number) {
    showIndicator(delta > 0 ? "volume_up" : "volume_down");
    await invoke("volume_control", { delta });
  }

  async function brightnessAction(delta: number) {
    showIndicator(delta > 0 ? "brightness_up" : "brightness_down");
    await invoke("brightness_control", { delta });
  }

  function showIndicator(type: string) {
    clearTimeout(gestureTimer);
    const labels: Record<string, string> = {
        "play_pause": "⏯ Toggle Play",
        "next": "⏭ Next Track",
        "prev": "⏮ Previous Track",
        "volume_up": "🔊 Volume Up",
        "volume_down": "🔉 Volume Down",
        "brightness_up": "🔆 Brightness Up",
        "brightness_down": "🔅 Brightness Down"
    };
    activeGesture = labels[type] || type;
    gestureTimer = setTimeout(() => { activeGesture = ""; }, 1500);
  }

  onMount(() => {
    window.addEventListener("gesture-play", () => mediaAction("play_pause"));
    window.addEventListener("gesture-pause", () => mediaAction("play_pause"));
    window.addEventListener("gesture-next", () => mediaAction("next"));
    window.addEventListener("gesture-prev", () => mediaAction("prev"));
  });
</script>

<main class="container">
  <h1>Aura</h1>
  <p class="subtitle">System Controller</p>
  
  <div class="gesture-section">
      <GestureController />
      
      <!-- Visual Feedback Overlay -->
      {#if activeGesture}
      <div class="gesture-indicator">
          {activeGesture}
      </div>
      {/if}
      
      <p class="hint">✋ Play • ✊ Pause • ☝️ Next • 🤏 Prev</p>
  </div>

  <section class="control-group">
    <h2>Media</h2>
    <div class="row">
      <button onclick={() => mediaAction("prev")} title="Previous Track">
        <span class="icon">⏮</span>
      </button>
      <button onclick={() => mediaAction("play_pause")} title="Play/Pause" class="primary">
        <span class="icon">⏯</span>
      </button>
      <button onclick={() => mediaAction("next")} title="Next Track">
        <span class="icon">⏭</span>
      </button>
    </div>
  </section>

  <section class="control-group">
    <h2>Volume</h2>
    <div class="row">
      <button onclick={() => volumeAction(-5)} title="Volume Down">
        <span class="icon">🔉</span>
      </button>
      <div class="label">System Audio</div>
      <button onclick={() => volumeAction(5)} title="Volume Up">
        <span class="icon">🔊</span>
      </button>
    </div>
  </section>

  <section class="control-group">
    <h2>Brightness</h2>
    <div class="row">
      <button onclick={() => brightnessAction(-5)} title="Brightness Down">
        <span class="icon">🔅</span>
      </button>
      <div class="label">Display</div>
      <button onclick={() => brightnessAction(5)} title="Brightness Up">
        <span class="icon">🔆</span>
      </button>
    </div>
  </section>
</main>

<style>
  :root {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    background-color: #0f0f0f;
    color: #e0e0e0;
  }

  .container {
    max-width: 500px;
    margin: 0 auto;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  h1 {
    font-size: 3rem;
    margin: 0;
    background: linear-gradient(45deg, #00d2ff, #3a7bd5);
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    font-weight: 800;
  }

  .subtitle {
    margin-top: -0.5rem;
    color: #888;
    font-size: 1rem;
    letter-spacing: 0.1rem;
    text-transform: uppercase;
  }
  
  .gesture-section {
      position: relative;
      text-align: center;
      margin-bottom: 1rem;
  }
  
  .gesture-indicator {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: rgba(58, 123, 213, 0.9);
      color: white;
      padding: 10px 20px;
      border-radius: 30px;
      font-weight: bold;
      font-size: 1.2rem;
      pointer-events: none;
      z-index: 100;
      box-shadow: 0 0 20px rgba(0,0,0,0.5);
      animation: pop-in 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  }

  @keyframes pop-in {
      0% { transform: translate(-50%, -50%) scale(0.5); opacity: 0; }
      100% { transform: translate(-50%, -50%) scale(1); opacity: 1; }
  }
  
  .hint {
      font-size: 0.8rem;
      color: #666;
      margin-top: 0.5rem;
  }

  .control-group {
    background: #1a1a1a;
    padding: 1.5rem;
    border-radius: 16px;
    border: 1px solid #333;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }

  h2 {
    font-size: 0.9rem;
    text-transform: uppercase;
    color: #666;
    margin-top: 0;
    margin-bottom: 1rem;
    letter-spacing: 0.05rem;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .label {
    flex-grow: 1;
    text-align: center;
    font-weight: 500;
    color: #aaa;
  }

  button {
    background: #2a2a2a;
    border: 1px solid #3a3a3a;
    color: white;
    width: 60px;
    height: 60px;
    border-radius: 12px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  button:hover {
    background: #333;
    border-color: #555;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  button:active {
    transform: translateY(0);
    background: #222;
  }

  button.primary {
    background: #3a7bd5;
    border-color: #4a8be5;
  }

  button.primary:hover {
    background: #4a8be5;
  }

  .icon {
    font-size: 1.5rem;
  }
</style>
