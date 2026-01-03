import { parseCookie, parseJWT, redirect } from "./api";

const session = parseCookie(document.cookie).get("session");
export const user: { fmname: string; exp: number } =
  session && parseJWT(session);

export const authenticated = (..._: unknown[]) => {
  if (new URL(window.location.href).searchParams.has("user")) {
    return;
  }
  if (user === undefined) {
    redirect("/login");
    return;
  }
};
