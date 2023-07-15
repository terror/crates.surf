<script>
  import _ from 'lodash';

  let query = '', data = {};

  const search = async (query) => {
    data = query.trim() !== '' ? await (await fetch(`/api/search?query=${query}`)).json() : {};
  };

  function highlight(text, query) {
    if (!query || !text) return text;

    return text
      .split(new RegExp(`(${_.escapeRegExp(query)})`, 'gi'))
      .map((part, i) =>
        part.toLowerCase() === query.toLowerCase() ?
          `<span class="bg-yellow-200">${part}</span>` :
          part
      ).join('');
  }

  $: search(query);
</script>

<main>
  <div class='m-5'>
    <p>crates.surf 🏄</p>
    <label for="search">Search for a crate</label>
    <input class='bg-slate-100 p-1 rounded-lg' type="text" id="search" bind:value={query} placeholder='just'>
    <div class='w-[50%]'>
    {#if data.response?.hits?.hits?.length}
      {#if data.time}
        <p class='p-1 my-1 rounded-lg bg-green-200'>Found {data.response.hits.hits.length} result(s) in {data.time}ms</p>
      {/if}
      <ul>
        {#each data.response.hits.hits as result (result._source.id)}
        <div class='bg-slate-100 p-1 my-1 rounded-lg'>
          <li>{@html highlight(result._source.name, query)}</li>
          <li>{@html highlight(result._source.description, query)}</li>
        </div>
        {/each}
      </ul>
    {/if}
    </div>
  <div>
</main>
