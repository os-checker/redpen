<template>
  <div v-motion :initial="{ x: -80, opacity: 0 }" :enter="val.enter" :leave="{ x: 100, opacity: 0 }" :style="val.style"
    class="absolute pointer-events-none border-2 border-amber-500 bg-amber-500/5 rounded-md" />
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  /** relative horizontal distance to this div */
  x: number,
  /** relative vertical distance to this div */
  y: number,
  /** rect width */
  w: number,
  /** rect height */
  h: number,
}>();

const val = computed(() => {
  const enter = { x: props.x, y: props.y, opacity: 1 };

  const style = { width: `${props.w}px`, height: `${props.h}px` };

  return { enter, style }
});
</script>
