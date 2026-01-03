<script>
  import { startNewGame } from "../../lib/api";
  import Game from "../../lib/Game.svelte";
</script>

<main class="p-16">
  <div class="flex justify-center items-center h-full">
    {#await startNewGame()}
      <p>loading game</p>
    {:then game}
      <Game initialLen={game.len} initialId={game.id} />
    {:catch}
      <p>failed to connect to server :(</p>
    {/await}
  </div>
</main>
