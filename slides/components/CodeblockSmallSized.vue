<template>
  <div class="codeblock-small" :style="{
    '--fs': fontSize,
    '--rg': rowGap
  }">
    <slot />
  </div>
</template>

<script setup>
defineProps({
  fontSize: { type: String, default: '12px' },
  rowGap: { type: String, default: '0.1rem' }
})
</script>

<style scoped>
.codeblock-small :deep(pre),
.codeblock-small :deep(pre code) {
  font-size: var(--fs) !important;
  row-gap: var(--rg) !important;
}
</style>
