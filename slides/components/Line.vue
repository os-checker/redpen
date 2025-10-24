<script setup lang="ts">
defineProps<{
  x1: string | number   // 起点 x
  y1: string | number   // 起点 y
  x2: string | number   // 终点 x
  y2?: string | number   // 终点 y
  color?: string        // 颜色
  width?: number        // 线宽
}>()
</script>

<template>
  <svg class="absolute inset-0 w-full h-full pointer-events-none" overflow="visible">
    <line :x1="x1" :y1="y1" :x2="x2" :y2="y2 ?? y1" :stroke="color ?? '#f43f5e'" :stroke-width="width ?? 2"
      stroke-linecap="round" filter="url(#rough)" />
  </svg>
</template>
