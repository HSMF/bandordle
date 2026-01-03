<script lang="ts">
  import { parseCookie, redirect, signin } from "./lib/api";

  import Home from "./pages/home/page.svelte";
  import Album from "./pages/album/page.svelte";
  import Artist from "./pages/artist/page.svelte";
  import Login from "./pages/login/page.svelte";
  import Track from "./pages/track/page.svelte";

  import page from "page";
  import { authenticated } from "./lib/user.svelte";

  let Page = $state(Home);
  let err: string | null = $state(null);

  page("/login", () => {
    if (parseCookie(document.cookie).has("session")) {
      redirect("/");
      return;
    }
    Page = Login;
  });
  page("/signin", () => {
    const urlParams = new URLSearchParams(window.location.search);
    const token = urlParams.get("token");
    if (token === null) {
      err = "no token";
      redirect("login");
    } else {
      signin(token);
    }
  });
  page("/", () => authenticated((Page = Home)));
  page("/album", () => authenticated((Page = Album)));
  page("/artist", () => authenticated((Page = Artist)));
  page("/track", () => authenticated((Page = Track)));
  page("*", () => {
    redirect("/");
  });
  page();
</script>

<svelte:head>
  <title>Bandordle</title>
</svelte:head>

<Page />

<style>
</style>
