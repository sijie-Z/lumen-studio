import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { Route, Router } from "@solidjs/router";
import { render } from "solid-js/web";
import "virtual:uno.css";
import "./styles/global.css";
import App from "./app";
import Home from "./pages/home";
import Login from "./pages/login";
import Register from "./pages/register";
import Dashboard from "./pages/dashboard";
import Explore from "./pages/explore";
import WorkDetail from "./pages/work-detail";
import Services from "./pages/services";
import ServiceDetail from "./pages/service-detail";
import CreatorProfile from "./pages/creator-profile";
import Admin from "./pages/admin";
import Account from "./pages/account";
import PublicLayout from "./components/layout/public-layout";
import CustomerLayout from "./components/layout/customer-layout";
import CreatorLayout from "./components/layout/creator-layout";
import AdminLayout from "./components/layout/admin-layout";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 30_000,
      refetchOnWindowFocus: false
    }
  }
});

const root = document.getElementById("root");
if (!root) throw new Error("Root element not found");

render(
  () => (
    <QueryClientProvider client={queryClient}>
      <Router root={App}>
        <Route component={PublicLayout}>
          <Route path="/" component={Home} />
          <Route path="/explore" component={Explore} />
          <Route path="/works/:id" component={WorkDetail} />
          <Route path="/services" component={Services} />
          <Route path="/services/:id" component={ServiceDetail} />
          <Route path="/creators/:id" component={CreatorProfile} />
          <Route path="/login" component={Login} />
          <Route path="/register" component={Register} />
        </Route>
        <Route component={CustomerLayout}>
          <Route path="/account" component={Account} />
        </Route>
        <Route component={CreatorLayout}>
          <Route path="/dashboard" component={Dashboard} />
        </Route>
        <Route component={AdminLayout}>
          <Route path="/admin" component={Admin} />
        </Route>
        <Route path="*" component={Home} />
      </Router>
    </QueryClientProvider>
  ),
  root
);
