<script lang="ts">
  import DropdownCheckbox from './DropdownCheckbox.svelte';
  import { settings, activeSubgraphs } from '$lib/stores/settings';

  $: dropdownOptions = Object.keys($settings?.subgraphs ?? {}).reduce(
    (acc, key) => ({
      ...acc,
      [key]: key,
    }),
    {},
  );

  function handleStatusChange(event: CustomEvent<Record<string, string>>) {
    let items = Object.keys(event.detail);
    activeSubgraphs.set(
      Object.values(items).reduce(
        (acc, key) => ({ ...acc, [key]: ($settings?.subgraphs ?? {})[key] }),
        {} as Record<string, string>,
      ),
    );
  }

  $: value =
    Object.keys($activeSubgraphs).length === 0
      ? {}
      : Object.keys($activeSubgraphs).reduce(
          (acc, key) => ({
            ...acc,
            [key]: key,
          }),
          {},
        );
</script>

<DropdownCheckbox
  options={dropdownOptions}
  on:change={handleStatusChange}
  label="Networks"
  showAllLabel={false}
  onlyTitle={true}
  {value}
/>
