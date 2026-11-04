import type { SVGProps } from "react";

const PATHS = {
  plus: <path d="M8 3v10M3 8h10" />,
  close: <path d="M4 4l8 8M12 4l-8 8" />,
  play: <path d="M5 3.5l7 4.5-7 4.5z" fill="currentColor" stroke="none" />,
  stop: <rect x="4.5" y="4.5" width="7" height="7" rx="1" fill="currentColor" stroke="none" />,
  gear: (
    <>
      <circle cx="8" cy="8" r="2.2" />
      <path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4" />
    </>
  ),
  search: (
    <>
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.3 10.3L14 14" />
    </>
  ),
  folder: <path d="M1.8 4.2h4.4l1.5 1.6h6.5v7.4H1.8z" />,
  check: <path d="M3 8.5l3.2 3L13 4.5" />,
  grip: (
    <>
      <circle cx="6" cy="4" r="0.9" fill="currentColor" stroke="none" />
      <circle cx="10" cy="4" r="0.9" fill="currentColor" stroke="none" />
      <circle cx="6" cy="8" r="0.9" fill="currentColor" stroke="none" />
      <circle cx="10" cy="8" r="0.9" fill="currentColor" stroke="none" />
      <circle cx="6" cy="12" r="0.9" fill="currentColor" stroke="none" />
      <circle cx="10" cy="12" r="0.9" fill="currentColor" stroke="none" />
    </>
  ),
  speaker: (
    <>
      <path d="M2.5 6h2.5l3.5-3v10L5 10H2.5z" />
      <path d="M11 5.5a3.2 3.2 0 0 1 0 5M12.8 3.6a6 6 0 0 1 0 8.8" />
    </>
  ),
  speakerOff: (
    <>
      <path d="M2.5 6h2.5l3.5-3v10L5 10H2.5z" />
      <path d="M11 6l3.5 4M14.5 6L11 10" />
    </>
  ),
  music: (
    <>
      <path d="M6 12V3.5l7-1.5v8.5" />
      <circle cx="4.5" cy="12" r="1.8" />
      <circle cx="11.5" cy="10.5" r="1.8" />
    </>
  ),
} as const;

export type IconName = keyof typeof PATHS;

interface IconProps extends Omit<SVGProps<SVGSVGElement>, "name"> {
  name: IconName;
  size?: number;
}

export function Icon({ name, size = 16, strokeWidth = 1.6, ...rest }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      {...rest}
    >
      {PATHS[name]}
    </svg>
  );
}
