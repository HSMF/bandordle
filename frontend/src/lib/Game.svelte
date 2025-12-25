<script lang="ts">
  import { twMerge } from "tailwind-merge";
  import { makeGuess } from "./api";
  import type { Grade } from "../../../bindings/Grade";

  const { initialId, initialLen } = $props();

  type Guess = {
    guess: string;
    grade: Grade[];
  };
  let previousGuesses: Guess[] = $state([]);

  // svelte-ignore state_referenced_locally
  let cells: (string | undefined)[] = $state(
    new Array(initialLen).fill(undefined),
  );
  let selectedCell = $state(0);

  function isLetter(ch: string) {
    return ch.length === 1 && ch.match(/[a-zA-Z]/);
  }

  function lockInGuess() {
    if (cells.includes(undefined)) {
      return;
    }

    const guess = cells.join("");
    makeGuess(initialId, guess).then(({ grade }) => {
      previousGuesses.push({
        grade,
        guess,
      });
      cells = new Array(initialLen).fill(undefined);
      selectedCell = 0;
    });
  }

  function gradeColor(grade: Grade | undefined) {
    switch (grade) {
      case undefined:
        return undefined;
      case "Incorrect":
        return "bg-gray-600";
      case "Correct":
        return "bg-green-600";
      case "WrongPlace":
        return "bg-yellow-600";
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    const key = event.key.toLowerCase();
    if (key === "backspace") {
      selectedCell = Math.max(selectedCell - 1, 0);
      cells[selectedCell] = undefined;
    } else if (key === "enter") {
      lockInGuess();
    } else if (isLetter(key) && selectedCell < cells.length) {
      cells[selectedCell] = key;
      selectedCell++;
    }
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
  <div class="flex flex-col gap-1">
    {#each previousGuesses as prev}
      <div class="flex gap-1">
        {#each prev.guess.split("") as ch, i}
          {@render Cell(ch, -1, () => {}, prev.grade[i])}
        {/each}
      </div>
    {/each}

    <div class="flex gap-1">
      {#each cells as cell, i}
        {@render Cell(cell, i, () => {
          selectedCell = i;
        })}
      {/each}
    </div>
  </div>
</div>
