import type { GuessArgs } from "../../../bindings/GuessArgs";
import type { GuessResult } from "../../../bindings/GuessResult";
import type { NewGameResult } from "../../../bindings/NewGameResult";

type ApiOpts = {
  query?: Record<string, string | null>;
};

const api = (u: string, conf?: ApiOpts) => {
  const ret = new URL(
    u,
    import.meta.env.VITE_BACKEND_URL ?? new URL(window.location.href).origin,
  );

  const query = conf?.query;
  if (query !== undefined) {
    for (const prop in query) {
      if (Object.hasOwn(query, prop)) {
        if (query[prop] === null) {
          continue;
        }
        ret.searchParams.append(prop, query[prop]);
      }
    }
  }

  return ret;
};

export type RedirectOpts = {
  force?: boolean;
};
export const redirect = (path: string, opts?: RedirectOpts) => {
  if (opts?.force) {
    window.location.href = path;
    return;
  }

  const u = new URL(window.location.href);
  u.pathname = path;
  window.location.href = u.toString();
  // window.history.pushState(null, "", path);
};

export const parseCookie = (cookie: string): Map<string, string> =>
  cookie.split(";").reduce((acc, v) => {
    const cookie = v.split("=");
    if (cookie.length !== 2) {
      return acc;
    }
    const key = decodeURIComponent(cookie[0].trim());
    const value = decodeURIComponent(cookie[1].trim());
    acc.set(key, value);
    return acc;
  }, new Map<string, string>());

export const parseJWT = (token: string) => {
  const base64Url = token.split(".")[1];
  const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/");
  const jsonPayload = decodeURIComponent(
    window
      .atob(base64)
      .split("")
      .map(function (c) {
        return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
      })
      .join(""),
  );

  return JSON.parse(jsonPayload);
};

export const startNewGame = async () => {
  const urlParams = new URLSearchParams(window.location.search);

  const user = urlParams.has("user") ? `?user=${urlParams.get("user")}` : "";
  const resp = await fetch(api(`/api/v1/newgame${user}`), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    credentials: "include",
  });
  if (!resp.ok) throw await resp.json();
  const body = await resp.json();
  return body as NewGameResult;
};

export const makeGuess = async (id: string, guess: string) => {
  const args: GuessArgs = { id, guess };
  const resp = await fetch(api(`/api/v1/guess`), {
    body: JSON.stringify(args),
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
  });
  if (!resp.ok) throw await resp.json();
  const body = await resp.json();
  return body as GuessResult;
};

export const signin = async (token: string) => {
  const resp = await fetch(api(`/api/v1/signin`, { query: { token } }), {
    credentials: "include",
  });
  if (resp.redirected) {
    const u = new URL(resp.url);
    redirect(u.pathname, { force: true });
  }
};

export const get_auth_url = async () => {
  const resp = await fetch(api(`/api/v1/auth-url`));
  if (!resp.ok) throw await resp.json();
  const body = await resp.text();

  return body as string;
};
