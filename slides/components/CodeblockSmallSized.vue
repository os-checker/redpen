<template>
  <div class="codeblock-small">
    <slot />
  </div>
</template>

<style lang="css" scoped>
.codeblock-small :deep(pre),
.codeblock-small :deep(pre code) {
  font-size: 12px !important;
  row-gap: 0.1rem !important;
}
</style>
