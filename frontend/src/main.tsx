import { Toast } from "@base-ui/react";
import { QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "react-router";

import { Toaster } from "@/components/toaster";
import { queryClient } from "@/lib/query-client";
import { toastManager } from "@/lib/toast";
import { router } from "@/router";

import "./index.css";

const MAX_VISIBLE_TOASTS = 3;

const rootElement = document.querySelector("#root");
if (!rootElement) throw new Error("Root element not found");

createRoot(rootElement).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <Toast.Provider toastManager={toastManager} limit={MAX_VISIBLE_TOASTS}>
        <RouterProvider router={router} />
        <Toaster />
      </Toast.Provider>
    </QueryClientProvider>
  </StrictMode>,
);
