<script lang="ts">
import _ from 'lodash';
import { onMount } from 'svelte';
import { writable } from 'svelte/store';

onMount(() =>
  data.subscribe((value) => {
    window.clearTimeout(showingTimeout);
    showing = 0;
    showMore();
  })
);

let data = writable({}),
  query = '',
  error = '';

const search = async (query: string) => {
  error = '';

  try {
    data.set(
      query.trim() !== ''
        ? await (
            await fetch(`/api/search?query=${encodeURIComponent(query)}`)
          ).json()
        : {}
    );
  } catch (err) {
    error = err.toString();
  }
};

const highlight = (text: string, query: string) => {
  return !query || !text
    ? text
    : text
        .split(new RegExp(`(${_.escapeRegExp(query)})`, 'gi'))
        .map((part, i) =>
          part.toLowerCase() === query.toLowerCase()
            ? `<span class="bg-yellow-200">${part}</span>`
            : part
        )
        .join('');
};

let showing = 0,
  showingTimeout = 0;

const showMore = () => {
  const len = $data?.response?.hits?.hits?.length ?? 0;

  if (showing < len) {
    showing += Math.min(20, len - showing);
    showingTimeout = window.setTimeout(showMore, 100);
  }
};

const activityBadge = (dateStr: string): boolean => {
  const date = new Date(dateStr);

  const now = new Date();

  return !((now - date) / (1000 * 3600 * 24 * 365) > 1)
    ? `<p class='bg-green-200 px-1 rounded-lg'>active</p>`
    : `<p class='bg-red-200 px-1 rounded-lg'>inactive</p>`;
};

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr);

  const months = [
    'January',
    'February',
    'March',
    'April',
    'May',
    'June',
    'July',
    'August',
    'September',
    'October',
    'November',
    'December',
  ];

  return `${months[date.getMonth()]} ${date.getDate()}, ${date.getFullYear()}`;
};

$: search(query);
</script>

<main class="m-auto w-[75%]">
  <div class="m-5">
    <a href="/" class="block"><strong>crates.surf 🏄</strong></a>
    <label for="search">Search for a crate</label>
    <input
      class="rounded-lg bg-slate-100 p-1"
      type="text"
      id="search"
      bind:value="{query}"
      placeholder="just"
    />
    <div>
      {#if error}<p class="my-1 rounded-lg bg-red-200 p-1">{error}</p>{/if}
      {#if $data?.response?.hits?.hits?.length}
        {#if $data.time}
          <p class="my-1 rounded-lg bg-green-200 p-1">
            Found {$data.response.hits.total.value} result(s) in {$data.time}ms
          </p>
          <ul>
            {#each $data.response.hits.hits.slice(0, showing) as result (result._source.id)}
              <div class="my-1 rounded-lg bg-slate-100 p-1">
                <li class="flex gap-x-2">
                  <strong
                    ><a
                      class="underline"
                      href="https://crates.io/crates/{result._source.name}"
                      target="_blank"
                      >{@html highlight(result._source.name, query)}</a
                    ></strong
                  >
                  {@html activityBadge(result._source.updated_at)}
                </li>
                <li>{@html highlight(result._source.description, query)}</li>
                <li>
                  Downloaded <strong>{result._source.downloads}</strong> times
                </li>
                <li>Created on {formatDate(result._source.created_at)}</li>
              </div>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  </div>
</main>
