---
# You can also start simply with 'default'
theme: seriph
# 'auto'，'light' or 'dark'
colorSchema: auto
# some information about your slides (markdown enabled)
title: "编写 Rust 静态分析工具"
titleTemplate: '%s'
info: |
  tag-std: https://github.com/Artisan-Lab/tag-asterinas
# apply unocss classes to the current slide
class: text-center
# https://sli.dev/features/drawing
drawings:
  persist: false
# slide transition: https://sli.dev/guide/animations.html#slide-transitions
transition: slide-left
# enable MDC Syntax: https://sli.dev/features/mdc
# mdc: true
# open graph
seoMeta:
  ogImage: https://cover.sli.dev
monaco: false
# controls whether texts in slides are selectable
# selectable: true
routerMode: hash
hideInToc: true
# download: true
---

<h1 style="font-size: 3.2rem">基于 Rust 编译器的静态分析工具：<br>基本流程与示例实现</h1>

<div class="text-xl font-bold">

仓库：[os-checker/redpen](https://github.com/os-checker/redpen)

分享者：周积萍

</div>

<style scoped>
.slidev-layout.cover {
  background: var(--slidev-theme-background) !important;
  color: var(--slidev-theme-foreground) !important;
}
</style>

---
hideInToc: true
routeAlias: toc
---

# 目录

<Toc maxDepth="1" />

---
src: ./pages/01-intro.md
---

---
src: ./pages/02-defid-hirid.md
---

---
src: ./pages/03-mir.md
---

---
src: ./pages/04-redpen-checker.md
---

---
src: ./pages/05-linter.md
---
