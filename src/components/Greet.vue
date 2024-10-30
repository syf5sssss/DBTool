<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  // greetMsg.value = await invoke("greet", { name: name.value });
  tauriCommand();
}

const tauriCommand = async () => {
  try {
    const response = await invoke('sqlx_connect_db');
    console.log(response);
    alert(response);
    greetMsg.value = response as string;
    const users = JSON.parse(response as string);
    console.log(users);
    alert(users);
    // 处理用户数据...
  } catch (error) {
    console.error('Error:', error);
    alert(error);
  }
};
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <input id="greet-input" v-model="name" placeholder="Enter a name..." />
    <button type="submit">Greet</button>
  </form>

  <p>{{ greetMsg }}</p>
</template>
