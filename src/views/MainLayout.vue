<template>
  <div class="main-layout">
    <Sidebar />
    <div class="main-content">
      <router-view />
    </div>
    <PlayerBar />
  </div>
</template>

<script setup lang="ts">
import Sidebar from "../components/Sidebar.vue";
import PlayerBar from "../components/PlayerBar.vue";
</script>

<style scoped>
.main-layout {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background-color: #0f172a;
  color: #e2e8f0;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}
</style>
