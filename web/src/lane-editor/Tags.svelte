<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { network } from "../osm2streets-svelte/store";

  const dispatch = createEventDispatcher<{
    editedWay: bigint;
  }>();

  // Immutable; use https://svelte.dev/tutorial/key-blocks around this entire
  // compoenent to change
  export let way: bigint;

  interface Tag {
    // Note IDs initially use indices, but as the user adds and deletes rows,
    // the indices get out of sync. That's not important; as long as the IDs
    // are unique, it's fine.
    id: number;
    key: string;
    value: string;
  }
  let tags: Tag[] = [];
  let uniqueID = 0;

  for (let [key, value] of Object.entries(
    JSON.parse($network!.getOsmTagsForWay(way))
  )) {
    tags.push({
      id: uniqueID++,
      key,
      value: value as string,
    });
  }

  function deleteTag(id: number) {
    tags = tags.filter((tag) => tag.id != id);
  }

  function addTag() {
    tags.push({
      id: uniqueID++,
      key: "",
      value: "",
    });
    tags = tags;
  }

  function recalculate() {
    let obj = {};
    for (let tag of tags) {
      // Skip empty keys and values
      if (tag.key && tag.value) {
        obj[tag.key] = tag.value;
        // TODO Warn for duplicate keys
      }
    }
    // TODO The user can cause a panic by passing invalid input, then everything stops working
    $network!.overwriteOsmTagsForWay(way, JSON.stringify(obj));
    $network = $network;

    dispatch("editedWay", way);
  }
</script>

<table>
  <tbody>
    {#each tags as tag (tag.id)}
      <tr>
        <td><input type="text" bind:value={tag.key} /></td>
        <td><input type="text" bind:value={tag.value} /></td>
        <td>
          <button type="button" on:click={() => deleteTag(tag.id)}
            >Delete</button
          >
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<button type="button" on:click={addTag}>Add new tag</button>
<button type="button" on:click={recalculate}>Recalculate</button>
