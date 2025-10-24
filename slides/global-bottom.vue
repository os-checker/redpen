<!-- an example footer for pages -->
<template>
  <footer v-if="$nav.currentLayout !== 'cover'" class="absolute bottom-0 right-4 p-2">
    {{ $nav.currentPage }} / {{ $nav.total }}
  </footer>
</template>
