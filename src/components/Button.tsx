import type { ButtonHTMLAttributes, ReactNode } from "react";

import { Icon, type IconName } from "./Icon";
import styles from "./Button.module.css";

type Variant = "primary" | "secondary" | "ghost" | "danger" | "dashed" | "link";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: IconName;
  children: ReactNode;
}

export function Button({ variant = "secondary", icon, children, className, ...rest }: ButtonProps) {
  return (
    <button type="button" className={`${styles.button} ${styles[variant]} ${className ?? ""}`} {...rest}>
      {icon && <Icon name={icon} size={14} />}
      {children}
    </button>
  );
}

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  icon: IconName;
  label: string;
  round?: boolean;
}

export function IconButton({ icon, label, round, className, ...rest }: IconButtonProps) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      className={`${styles.iconButton} ${round ? styles.round : ""} ${className ?? ""}`}
      {...rest}
    >
      <Icon name={icon} size={round ? 11 : 13} strokeWidth={1.8} />
    </button>
  );
}
