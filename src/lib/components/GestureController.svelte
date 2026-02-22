<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as handTrack from 'handtrackjs';

  let video: HTMLVideoElement;
  let canvas: HTMLCanvasElement;
  let model: any = null;
  let webcamRunning: boolean = $state(false);
  let modelLoading: boolean = $state(true);
  let errorMessage: string = $state("");
  
  const modelParams = {
    flipHorizontal: true,
    maxNumBoxes: 1,
    iouThreshold: 0.5,
    scoreThreshold: 0.6,
  };

  const dispatch = (eventName: string) => {
    window.dispatchEvent(new CustomEvent(eventName));
  };

  onMount(async () => {
    try {
      model = await handTrack.load(modelParams);
      modelLoading = false;
    } catch (e) {
      errorMessage = "Failed to load AI model.";
      console.error(e);
    }
  });

  onDestroy(() => {
    stopCamera();
  });

  async function startCamera() {
    errorMessage = "";
    if (!model) return;
    
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ 
          video: { width: 320, height: 180 } 
      });
      
      if (video) {
        video.srcObject = stream;
        video.onloadedmetadata = () => {
          video.play();
          webcamRunning = true;
          runDetection();
        };
      }
    } catch (err: any) {
      console.error("Camera Error:", err);
      if (err.name === "NotAllowedError") {
          errorMessage = "Camera permission denied.";
      } else if (err.name === "NotFoundError") {
          errorMessage = "No camera found.";
      } else {
          errorMessage = "Camera error: " + err.message;
      }
    }
  }

  function stopCamera() {
    webcamRunning = false;
    if (video && video.srcObject) {
      const stream = video.srcObject as MediaStream;
      stream.getTracks().forEach(track => track.stop());
      video.srcObject = null;
    }
  }

  let lastLabel = "";
  let lastTime = 0;

  async function runDetection() {
    if (!webcamRunning || !model) return;

    try {
        const predictions = await model.detect(video);
        const context = canvas.getContext("2d");
        if (context) {
            context.clearRect(0, 0, canvas.width, canvas.height);
            model.renderPredictions(predictions, canvas, context, video);
        }

        if (predictions.length > 0) {
            const label = predictions[0].label;
            const now = Date.now();

            if (label !== lastLabel && (now - lastTime > 1500)) {
                if (label === "open") dispatch("gesture-play");
                else if (label === "closed") dispatch("gesture-pause");
                else if (label === "point") dispatch("gesture-next");
                else if (label === "pinch") dispatch("gesture-prev");
                
                lastLabel = label;
                lastTime = now;
            }
        }
    } catch (e) {
        console.error("Detection error:", e);
    }

    if (webcamRunning) {
      requestAnimationFrame(runDetection);
    }
  }
</script>

<div class="gesture-container">
  <!-- svelte-ignore a11y_media_has_caption -->
  <video bind:this={video} class="canvasbox" playsinline muted></video>
  <canvas bind:this={canvas} class="canvasbox" width="320" height="180"></canvas>
  
  <div class="overlay">
    {#if errorMessage}
      <div class="error">{errorMessage}</div>
      <button onclick={startCamera} class="start-btn mini">Retry</button>
    {:else if modelLoading}
      <div class="loading">Loading AI Model...</div>
    {:else if !webcamRunning}
      <button onclick={startCamera} class="start-btn">Enable Camera</button>
    {:else}
      <div class="status-tag">Live Recognition</div>
      <button onclick={stopCamera} class="stop-btn">Disable</button>
    {/if}
  </div>
</div>

<style>
  .gesture-container {
    position: relative;
    width: 320px;
    height: 180px;
    border-radius: 12px;
    overflow: hidden;
    background: #111;
    margin: 0 auto;
    border: 2px solid #333;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  video, canvas {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      object-fit: cover;
      pointer-events: none;
  }
  
  video {
      opacity: 0.5;
      transform: scaleX(-1); /* Mirror effect */
  }
  
  canvas {
      z-index: 10;
      transform: scaleX(-1); /* Mirror effect for landmarks too */
  }
  
  .overlay {
      z-index: 20;
      text-align: center;
      padding: 20px;
  }

  .start-btn {
      background: #3a7bd5;
      color: white;
      border: none;
      padding: 12px 24px;
      border-radius: 8px;
      cursor: pointer;
      font-weight: bold;
      box-shadow: 0 4px 15px rgba(0,0,0,0.4);
      transition: background 0.2s;
  }
  
  .start-btn:hover {
      background: #4a8be5;
  }
  
  .start-btn.mini {
      padding: 6px 12px;
      margin-top: 10px;
      font-size: 12px;
  }

  .stop-btn {
      position: absolute;
      top: 10px;
      right: 10px;
      background: rgba(255,0,0,0.3);
      color: white;
      border: 1px solid rgba(255,0,0,0.2);
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 10px;
      cursor: pointer;
  }

  .status-tag {
      position: absolute;
      bottom: 10px;
      left: 10px;
      background: rgba(0,255,0,0.1);
      color: #0f0;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 10px;
      border: 1px solid rgba(0,255,0,0.2);
  }

  .loading {
      color: #888;
      font-size: 14px;
      font-weight: 500;
  }
  
  .error {
      color: #ff4444;
      font-size: 14px;
      margin-bottom: 5px;
      background: rgba(0,0,0,0.5);
      padding: 5px 10px;
      border-radius: 4px;
  }
</style>
