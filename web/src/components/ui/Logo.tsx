export interface LogoProps {
  size?: number;
  className?: string;
}

export function Logo({ size = 32, className }: LogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <circle cx="11" cy="11" r="8" fill="#E8847A" opacity="0.8" />
      <circle cx="21" cy="11" r="8" fill="#5EC99A" opacity="0.8" />
      <circle cx="16" cy="19" r="8" fill="#5A9FD4" opacity="0.8" />
    </svg>
  );
}
