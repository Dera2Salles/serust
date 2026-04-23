import React from 'react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export const Card = ({ children, className, ...props }: CardProps) => (
  <div 
    className={cn("oneui-card", className)}
    {...props}
  >
    {children}
  </div>
);

interface HeaderProps {
  title: string;
  subtitle?: string;
  className?: string;
}

export const Header = ({ title, subtitle, className }: HeaderProps) => (
  <div className={cn("oneui-header", className)}>
    <h1 className="text-5xl font-light tracking-tight mb-2">{title}</h1>
    {subtitle && <p className="text-subtext0 text-lg">{subtitle}</p>}
  </div>
);

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary';
}

export const Button = ({ variant = 'primary', className, children, ...props }: ButtonProps) => (
  <button 
    className={cn(
      variant === 'primary' ? 'oneui-button-primary' : 'bg-surface1 text-text px-6 py-3 rounded-full active:scale-95',
      className
    )}
    {...props}
  >
    {children}
  </button>
);
