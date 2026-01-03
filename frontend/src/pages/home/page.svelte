<script lang="ts">
  const url = (path: string) => {
    const ret = new URL(window.location.href);
    ret.pathname = path;
    return ret;
  }
</script>


<main class="flex justify-center p-24">
  <div class="flex flex-col items-center">
    <h1 class="text-2xl">Bandordle</h1>

    <ul>
      <li><a class="underline" href={url("/artist")}>guess the artist</a></li>
      <li><a class="underline" href="/#">guess the album (TODO)</a></li>
    </ul>
  </div>
</main>
