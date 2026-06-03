import React from 'react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/* ─── FluentCard ────────────────────────────────────────────── */
interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}
export const SoftCard = ({ children, className, ...props }: CardProps) => (
  <div className={cn('fluent-card', className)} {...props}>{children}</div>
);
export const Card = SoftCard;

/* ─── Page Header ───────────────────────────────────────────── */
interface HeaderProps { title: string; subtitle?: string; className?: string; }
export const Header = ({ title, subtitle, className }: HeaderProps) => (
  <div className={cn('px-8 pt-8 pb-6', className)}>
    <h1 style={{ fontSize: 28, fontWeight: 600, color: 'var(--color-win-text)', letterSpacing: '-0.3px', lineHeight: 1.2, marginBottom: subtitle ? 4 : 0 }}>
      {title}
    </h1>
    {subtitle && (
      <p style={{ fontSize: 13, color: 'var(--color-win-text3)', fontWeight: 400 }}>{subtitle}</p>
    )}
  </div>
);

/* ─── Button ─────────────────────────────────────────────────── */
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'danger';
  icon?: React.ReactNode;
}
export const Button = ({ variant = 'primary', icon, className, children, ...props }: ButtonProps) => {
  const cls = {
    primary:   'fluent-btn fluent-btn-accent',
    secondary: 'fluent-btn',
    outline:   'fluent-btn',
    danger:    'fluent-btn fluent-btn-danger',
  }[variant];
  return (
    <button className={cn(cls, className)} {...props}>
      {icon && <span className="inline-flex flex-shrink-0">{icon}</span>}
      {children}
    </button>
  );
};

/* ─── TextField ──────────────────────────────────────────────── */
interface TextFieldProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  prefixIcon?: React.ReactNode;
}
export const ModernTextField = ({ label, prefixIcon, className, ...props }: TextFieldProps) => (
  <div className="flex flex-col gap-1.5 w-full">
    {label && (
      <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text2)' }}>{label}</label>
    )}
    <div className="relative">
      {prefixIcon && (
        <div className="absolute left-3 top-1/2 -translate-y-1/2" style={{ color: 'var(--color-win-text3)' }}>
          {prefixIcon}
        </div>
      )}
      <input
        className={cn('fluent-input', prefixIcon ? 'pl-9' : '', className)}
        {...props}
      />
    </div>
  </div>
);

/* ─── Badge / Chip ──────────────────────────────────────────── */
interface ChipProps { label: string; color?: string; className?: string; }
export const AroChip = ({ label, color = 'blue', className }: ChipProps) => {
  const cls: Record<string, string> = {
    blue:    'fluent-badge fluent-badge-blue',
    green:   'fluent-badge fluent-badge-green',
    red:     'fluent-badge fluent-badge-red',
    yellow:  'fluent-badge fluent-badge-warn',
    peach:   'fluent-badge fluent-badge-warn',
    mauve:   'fluent-badge fluent-badge-blue',
    overlay2:'fluent-badge fluent-badge-gray',
  };
  return <span className={cn(cls[color] || cls.blue, className)}>{label}</span>;
};

/* ─── Icon Button ───────────────────────────────────────────── */
interface IconBtnProps extends React.ButtonHTMLAttributes<HTMLButtonElement> { icon: React.ReactNode; }
export const AroIconButton = ({ icon, className, ...props }: IconBtnProps) => (
  <button
    className={cn('fluent-btn', 'px-2.5', className)}
    style={{ minWidth: 32, height: 32, padding: '0 8px', display: 'inline-flex', alignItems: 'center', justifyContent: 'center' }}
    {...props}
  >
    {icon}
  </button>
);
