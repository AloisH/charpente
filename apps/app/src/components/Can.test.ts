import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { defineComponent, h } from "vue";

import { meQueryKey } from "@charpente/api-client";
import type { UserDto } from "@charpente/api-client";

import Can from "@/components/Can.vue";

const Probe = defineComponent({
  render: () => h("span", { id: "probe" }, "visible"),
});

function mountWithUser(role: UserDto["role"] | null) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  if (role !== null) {
    const user: UserDto = {
      id: "0198b000-0000-7000-8000-000000000000",
      email: "t@example.com",
      display_name: "T",
      role,
      email_verified: true,
      created_at: new Date().toISOString(),
    };
    queryClient.setQueryData(meQueryKey(), user);
  }
  return mount(Can, {
    props: { permission: "manage-users" as const },
    slots: { default: Probe },
    global: {
      plugins: [[VueQueryPlugin, { queryClient }]],
      mocks: { $router: {} },
    },
  });
}

describe("Can", () => {
  it("renders the slot for a permitted role", () => {
    const wrapper = mountWithUser("admin");
    expect(wrapper.find("#probe").exists()).toBe(true);
  });

  it("hides the slot for a plain user", () => {
    const wrapper = mountWithUser("user");
    expect(wrapper.find("#probe").exists()).toBe(false);
  });

  it("hides the slot when anonymous", () => {
    const wrapper = mountWithUser(null);
    expect(wrapper.find("#probe").exists()).toBe(false);
  });
});
