import type { GuessArgs } from "../../../bindings/GuessArgs";
import type { GuessResult } from "../../../bindings/GuessResult";
import type { NewGameResult } from "../../../bindings/NewGameResult";

export const startNewGame = async () => {
  const resp = await fetch(
    `${import.meta.env.VITE_BACKEND_URL}/api/v1/newgame`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
    },
  );
  if (!resp.ok) throw await resp.json();
  const body = await resp.json();
  return body as NewGameResult;
};

export const makeGuess = async (id: string, guess: string) => {
  const args: GuessArgs = { id, guess };
  const resp = await fetch(`${import.meta.env.VITE_BACKEND_URL}/api/v1/guess`, {
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
