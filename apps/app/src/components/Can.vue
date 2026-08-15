<script setup lang="ts">
// Renders its slot only when the current user holds the permission:
//   <Can permission="manage-users"><AdminPanel /></Can>
import { useAuth, type Permission } from "@/composables/useAuth";

const props = defineProps<{ permission: Permission }>();
const { can } = useAuth();
</script>

<template>
  <slot v-if="can(props.permission)" />
</template>
