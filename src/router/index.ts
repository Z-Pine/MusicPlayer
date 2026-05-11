import { createRouter, createWebHashHistory } from "vue-router";
import MainLayout from "../views/MainLayout.vue";

const routes = [
  {
    path: "/",
    component: MainLayout,
    children: [
      {
        path: "",
        name: "library",
        component: () => import("../views/LibraryView.vue"),
      },
      {
        path: "playlist/:id",
        name: "playlist",
        component: () => import("../views/PlaylistView.vue"),
      },
      {
        path: "lyrics",
        name: "lyrics",
        component: () => import("../views/LyricsView.vue"),
      },
      {
        path: "settings",
        name: "settings",
        component: () => import("../views/SettingsView.vue"),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
