<script lang="ts">
  export let message: string;
  export let type: "success" | "error";
</script>

<div class="toast" class:success={type === "success"} class:error={type === "error"}>
  {message}
</div>

<style>
  .toast {
    position: fixed;
    top: 20px;
    right: 20px;
    padding: 15px 25px;
    border-radius: 8px;
    color: white;
    font-weight: 500;
    animation: slideIn 0.3s ease;
    z-index: 1000;
  }

  .success {
    background-color: #22c55e;
  }

  .error {
    background-color: #ef4444;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
</style>
