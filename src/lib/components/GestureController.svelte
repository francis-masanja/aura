<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Hands, HAND_CONNECTIONS } from '@mediapipe/hands';
  import { Camera } from '@mediapipe/camera_utils';

  let video: HTMLVideoElement;
  let canvas: HTMLCanvasElement;
  let canvasCtx: CanvasRenderingContext2D | null = null;
  let camera: Camera | null = null;
  let webcamRunning: boolean = $state(false);
  let modelLoading: boolean = $state(true);
  let errorMessage: string = $state("");
  let candidateLabel: string = $state("");

  let lastLabel = "";
  let lastTime = 0;
  let gestureCount = 0;
  const CONFIRMATION_FRAMES = 3;

  const dispatch = (eventName: string) => {
    window.dispatchEvent(new CustomEvent(eventName));
  };

  onMount(async () => {
    try {
      const hands = new Hands({
        locateFile: (file) => `https://cdn.jsdelivr.net/npm/@mediapipe/hands/${file}`
      });

      hands.setOptions({
        maxNumHands: 1,
        modelComplexity: 1,
        minDetectionConfidence: 0.7,
        minTrackingConfidence: 0.7
      });

      hands.onResults(onResults);
      
      // Initialize canvas context
      if (canvas) {
        canvasCtx = canvas.getContext('2d');
      }

      // Setup camera
      camera = new Camera(video, {
        onFrame: async () => {
          if (video && video.readyState >= 2) {
            await hands.send({ image: video });
          }
        },
        width: 320,
        height: 180
      });

      modelLoading = false;
      console.log("MediaPipe Hands loaded");
    } catch (e) {
      errorMessage = "Failed to load AI model.";
      console.error("Model loading error:", e);
      modelLoading = false;
    }
  });

  onDestroy(() => {
    stopCamera();
  });

  function onResults(results: any) {
    if (!canvasCtx || !canvas) return;

    canvas.width = video.videoWidth || 320;
    canvas.height = video.videoHeight || 180;
    canvasCtx.save();
    canvasCtx.clearRect(0, 0, canvas.width, canvas.height);

    if (results.multiHandLandmarks && results.multiHandLandmarks.length > 0) {
      const landmarks = results.multiHandLandmarks[0];
      
      // Draw hand landmarks
      // Note: We'll draw simple circles for landmarks
      for (const landmark of landmarks) {
        const x = landmark.x * canvas.width;
        const y = landmark.y * canvas.height;
        canvasCtx.beginPath();
        canvasCtx.arc(x, y, 3, 0, 2 * Math.PI);
        canvasCtx.fillStyle = '#10b981';
        canvasCtx.fill();
      }

      // Draw connections
      canvasCtx.strokeStyle = '#3b82f6';
      canvasCtx.lineWidth = 2;
      for (const [i, j] of HAND_CONNECTIONS) {
        const p1 = landmarks[i];
        const p2 = landmarks[j];
        canvasCtx.beginPath();
        canvasCtx.moveTo(p1.x * canvas.width, p1.y * canvas.height);
        canvasCtx.lineTo(p2.x * canvas.width, p2.y * canvas.height);
        canvasCtx.stroke();
      }

      processGestures(landmarks);
    } else {
      candidateLabel = "";
      gestureCount = 0;
    }
    canvasCtx.restore();
  }

  function processGestures(lm: any[]) {
    // Get landmark positions
    const thumb = lm[4];
    const index = lm[8];
    const middle = lm[12];
    const ring = lm[16];
    const pinky = lm[20];
    
    // Bases
    const iBase = lm[5];
    const mBase = lm[9];
    const rBase = lm[13];
    const pBase = lm[17];
    const wrist = lm[0];

    // Check finger states
    const indexUp = index.y < iBase.y;
    const middleUp = middle.y < mBase.y;
    const ringUp = ring.y < rBase.y;
    const pinkyUp = pinky.y < pBase.y;
    const thumbUp = thumb.y < lm[2].y;

    let detected = "";

    // Gesture detection logic
    if (indexUp && middleUp && ringUp && pinkyUp) {
      detected = "OPEN";
    } 
    else if (!indexUp && !middleUp && !ringUp && !pinkyUp && !thumbUp) {
      detected = "FIST";
    }
    else if (thumbUp && !indexUp && !middleUp && !ringUp) {
      detected = thumb.y < wrist.y ? "THUMBS UP" : "THUMBS DOWN";
    }
    else if (indexUp && !middleUp && !ringUp && !pinkyUp) {
      detected = "POINT";
    }

    if (detected) {
      const now = Date.now();
      
      if (detected === candidateLabel) {
        gestureCount++;
      } else {
        candidateLabel = detected;
        gestureCount = 0;
      }

      if (gestureCount >= CONFIRMATION_FRAMES) {
        if (detected !== lastLabel && (now - lastTime > 800)) {
          if (detected === "OPEN") {
            dispatch("gesture-play");
            console.log("Gesture: PLAY");
          } else if (detected === "FIST") {
            dispatch("gesture-pause");
            console.log("Gesture: PAUSE");
          } else if (detected === "POINT") {
            dispatch("gesture-next");
            console.log("Gesture: NEXT");
          } else if (detected === "THUMBS UP") {
            dispatch("gesture-prev");
            console.log("Gesture: PREV");
          }
          
          lastLabel = detected;
          lastTime = now;
        }
      }
    }
  }

  async function startCamera() {
    errorMessage = "";
    if (modelLoading) {
      errorMessage = "AI Model not loaded yet.";
      return;
    }
    
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ 
        video: { width: 320, height: 180 } 
      });
      
      if (video) {
        video.srcObject = stream;
        await video.play();
        webcamRunning = true;
        camera?.start();
      }
    } catch (err: any) {
      console.error("Camera Access Error:", err);
      if (err.name === "NotAllowedError") {
        errorMessage = "Camera permission denied.";
      } else if (err.name === "NotFoundError") {
        errorMessage = "No camera detected.";
      } else {
        errorMessage = "Could not access camera.";
      }
    }
  }

  function stopCamera() {
    webcamRunning = false;
    camera?.stop();
    if (video && video.srcObject) {
      const stream = video.srcObject as MediaStream;
      stream.getTracks().forEach(track => track.stop());
      video.srcObject = null;
    }
  }
</script>

<div class="gesture-container">
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
      <div class="status-tag">Live: {candidateLabel || "Searching..."}</div>
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
      transform: scaleX(-1);
  }
  
  canvas {
      z-index: 10;
      transform: scaleX(-1);
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
