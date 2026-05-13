<script>
  import "../app.css";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import LoginGate from "$lib/components/LoginGate.svelte";

  let { children } = $props();
</script>

{@render children()}
<ToastContainer />
<LoginGate />
