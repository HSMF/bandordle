<script>
  import { get_auth_url } from "../../lib/api";

  let user = $state("");
</script>

<main class="w-full h-dvh flex items-center justify-center">
  {#await get_auth_url()}
    <div>fetching redirect url</div>
  {:then value}
    <div class="flex flex-col">
      <a class="underline" href={value}>Login Via Last.fm</a>

      <label
        >or continue as user <input
          type="text"
          class="outline rounded-xl px-2"
          bind:value={user}
        /></label
      >
      {#if user}
        <a href={`/?user=${user}`}>continue as {user}</a>
      {/if}
    </div>
  {:catch err}
    <div>{err}</div>
  {/await}
</main>
