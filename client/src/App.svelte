<script lang="ts">
  import { onMount } from 'svelte';
  import { writable } from 'svelte/store';

  let search = '', data = {};

  const fetchResults = async (query) => {
    if (query.trim() !== '') {
      const response = await fetch(`/api/search?query=${query}`);
      data = await response.json();
      console.log(data);
    } else {
      data = {};
    }
  };

  $: fetchResults(search);
</script>

<main>
  <div class='m-5'>
    <p>crates.surf 🏄</p>
    <label for="search">Search for a crate</label>
    <input class='bg-slate-100 p-1 rounded-lg' type="text" id="search" bind:value={search} placeholder='just'>
    <div class='w-[50%]'>
    {#if data.response?.hits?.hits?.length}
      {#if data.time}
        <p class='p-1 my-1 rounded-lg bg-green-200'>Found {data.response.hits.hits.length} result(s) in {data.time}ms</p>
      {/if}
      <ul>
        {#each data.response.hits.hits as result (result._source.id)}
        <div class='bg-slate-100 p-1 my-1 rounded-lg'>
          <li>{result._source.name}</li>
          <li>{result._source.description}</li>
        </div>
        {/each}
      </ul>
    {/if}
    </div>
  <div>
</main>
