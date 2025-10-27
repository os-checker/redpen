# os-checker 与静态检查工具组件化

<Toc mode="onlyCurrentTree" />

<BackToTOC />

---

## os-checker


<div class="font-bold text-xl pt-6">

针对社区和学术界的各种代码分析和检查工具，提供统一的流程来使用它们，让使用者只需要关注分析和检查结果。

使代码检查流程化、自动化、规范化，最终通过 WebUI 一目了然地看到分析和检查结果，从而全方面地提高 Rust 项目的质量。

主页：<https://os-checker.github.io/>

完整介绍：<https://docs.qq.com/slide/DTEV3Z3pIbmF3eEhG>

</div>

---

## os-checker 集成的工具

![](https://github.com/user-attachments/assets/62b82eff-3e1a-49ae-a20f-0ad6932b0a10)

<style> p { margin: 0; } </style>

---
clicks: 1
---

## 诊断检查结果统计与查询

<script setup>
import { computed } from 'vue'
const { nav } = $slidev

const list = [
  'https://github.com/user-attachments/assets/3ca5ec46-68b7-4197-8aa4-5a87a5650ed1',
  'https://github.com/user-attachments/assets/cb602ec6-a5ab-4655-a4ea-1f32225f1e0c'
]

const currentImage = computed(() => {
  return list[nav.clicks % list.length]
})
</script>

<div @click="nav.next()" class="flex items-center justify-center h-100">
  <img :src="currentImage" class="max-h-full w-auto object-contain">
</div>

<div class="text-center">

[os-checker.github.io/diagnostics](https://os-checker.github.io/diagnostics)

</div>

---

## 静态检查工具组件化

<div class="absolute top-12 left-84">

[文档](https://os-checker.github.io/book/goal/componentization.html)

</div>

![](https://github.com/user-attachments/assets/3a192353-4ba6-4e9c-ba6d-8e2a5f099334)

<style> p { margin: 0; } </style>
