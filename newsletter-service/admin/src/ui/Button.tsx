import type { ButtonHTMLAttributes } from "react";

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "ghost";
  size?: "sm" | "md";
};

export function Button({ variant = "secondary", size = "md", className = "", ...rest }: Props) {
  return <button className={`ui-btn ui-btn--${variant} ui-btn--${size} ${className}`.trim()} {...rest} />;
}
