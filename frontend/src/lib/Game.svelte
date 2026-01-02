<script lang="ts">
  import { twMerge } from "tailwind-merge";
  import { makeGuess } from "./api";
  import type { Grade } from "../../../bindings/Grade";

  const { initialId, initialLen } = $props();

  type GuessedWord = {
    guess: string;
    grade: Grade[];
  };
  type Guess = GuessedWord[];
  let previousGuesses: Guess[] = $state([]);

  let error: string | undefined = $state(undefined);

  function newArr(lengths: number[]) {
    return lengths.map((l: number) => new Array(l).fill(undefined));
  }

  // svelte-ignore state_referenced_locally
  let cells: (string | undefined)[][] = $state(newArr(initialLen));
  let selectedCell = $state(0);

  function allowedChar(ch: string) {
    return ch.length === 1 && ch.match(/[a-zA-Z1-9]/);
  }

  function lockInGuess() {
    if (cells.findIndex((x) => x.includes(undefined)) >= 0) {
      return;
    }

    const guess = cells.map((x) => x.join("")).join(" ");
    makeGuess(initialId, guess)
    .then(({ grade }) => {
      const splitGuess = guess.split(/\s+/);
      previousGuesses.push(
        grade.map((grade, i) => {
          const guess = splitGuess[i];
          return { grade, guess };
        }),
      );

      cells = newArr(initialLen);
      selectedCell = 0;
      error = undefined
    })
    .catch((x) => { error = x.message ?? "oh no" })
    ;
  }

  function gradeColor(grade: Grade | undefined) {
    switch (grade) {
      case undefined:
        return undefined;
      case "Incorrect":
        return "bg-gray-600 text-white";
      case "Correct":
        return "bg-green-600";
      case "WrongPlace":
        return "bg-yellow-600";
    }
  }

  function effectiveIndex(wordIdx: number, inWordIdx: number) {
    const before = cells.slice(0, wordIdx);
    return before.map((x) => x.length).reduce((a, b) => a + b, 0) + inWordIdx;
  }

  function actualIndex(sel: number) {
    let i = 0;
    while (sel > 0 && i < cells.length) {
      if (sel < cells[i].length) {
        return [i, sel];
      }
      sel -= cells[i].length;
      i++;
    }
    return [i, sel];
  }
</script>

<svelte:window
  onkeydown={(event) => {
    const key = event.key.toLowerCase();
    if (key === "backspace") {
      selectedCell = Math.max(selectedCell - 1, 0);
      const [wordIdx, i] = actualIndex(selectedCell);
      cells[wordIdx][i] = undefined;
      return;
    }
    if (key === "enter") {
      lockInGuess();
      return;
    }
    const [wordIdx, i] = actualIndex(selectedCell);
    if (
      allowedChar(key) &&
      wordIdx < cells.length &&
      i < cells[wordIdx].length
    ) {
      cells[wordIdx][i] = key;
      selectedCell++;
    }
    console.log({ selectedCell });
  }}
/>

{#snippet Cell(
  content: string | undefined,
  i: number,
  onclick: () => void,
  grade?: Grade,
)}
  <button
    class={twMerge(
      "border-black border-2 min-w-12 aspect-square rounded-md",
      i === selectedCell && "border-yellow-500",
      gradeColor(grade),
    )}
    {onclick}
  >
    {content}
  </button>
{/snippet}

<div>

  {#if error !== undefined}
    <div class="text-red-500">{error}</div>
  {/if}

  <div class="flex flex-col gap-1">
    {#each previousGuesses as prev}
      <div class="flex gap-8">
        {#each prev as word}
          <div class="flex gap-1">
            {#each word.guess.split("") as ch, i}
              {@render Cell(ch, -1, () => {}, word.grade[i])}
            {/each}
          </div>
        {/each}
      </div>
    {/each}

    <div class="flex gap-8">
      {#each cells as word, wordIdx}
        <div class="flex gap-1">
          {#each word as cell, i}
            {@render Cell(cell, effectiveIndex(wordIdx, i), () => {
              selectedCell = effectiveIndex(wordIdx, i);
            })}
          {/each}
        </div>
      {/each}
    </div>
  </div>
</div>
