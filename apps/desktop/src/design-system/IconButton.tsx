import type { ButtonHTMLAttributes, ReactNode } from "react";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  label: string;
  icon: ReactNode;
}

export function IconButton({ className = "", icon, label, type = "button", ...props }: IconButtonProps) {
  return (
    <button aria-label={label} className={`icon-button ${className}`} title={label} type={type} {...props}>
      {icon}
    </button>
  );
}
