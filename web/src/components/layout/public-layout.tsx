import type { RouteSectionProps } from "@solidjs/router";
import SiteHeader from "./site-header";

export default function PublicLayout(props: RouteSectionProps) {
  return (
    <>
      <SiteHeader />
      {props.children}
    </>
  );
}
