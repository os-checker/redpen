<script setup lang="ts">
defineProps<{
  left?: string
  right?: string
  gap?: string
}>()
</script>

<template>
  <div :style="{ display: 'flex', gap: gap ?? '1rem' }">
    <div :style="{ flex: `1 1 ${left ?? '50%'}` }">
      <slot name="left" />
    </div>
    <div :style="{ flex: `1 1 ${right ?? '50%'}` }">
      <slot name="right" />
    </div>
  </div>
</template>
