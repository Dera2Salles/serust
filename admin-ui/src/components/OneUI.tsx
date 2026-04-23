import React from 'react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Utility for Tailwind class merging.
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * SoftCard - Radius 32px, mantle background, subtle border.
 * Matches Flutter's SoftCard.
 */
interface SoftCardProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export const SoftCard = ({ children, className, ...props }: SoftCardProps) => (
  <div 
    className={cn("oneui-card", className)}
    {...props}
  >
    {children}
  </div>
);

// Alias for backwards compatibility if needed, but we should prefer SoftCard
export const Card = SoftCard;

/**
 * Header - Large light tracking-tight titles.
 * Matches Flutter's screen headers.
 */
interface HeaderProps {
  title: string;
  subtitle?: string;
  className?: string;
}

export const Header = ({ title, subtitle, className }: HeaderProps) => (
  <div className={cn("oneui-header", className)}>
    <h1 className="text-5xl font-extrabold tracking-tighter mb-3 leading-tight">{title}</h1>
    {subtitle && <p className="text-subtext0 text-xl font-medium tracking-tight opacity-80">{subtitle}</p>}
  </div>
);

/**
 * GradientButton - Radius 100, scale animation, blue primary.
 * Matches Flutter's GradientButton.
 */
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline';
  icon?: React.ReactNode;
}

export const Button = ({ variant = 'primary', icon, className, children, ...props }: ButtonProps) => {
  const variantClasses = {
    primary: 'oneui-button-primary',
    secondary: 'bg-surface1 text-text hover:bg-surface2 px-6 py-3 rounded-full active:scale-95 transition-all',
    outline: 'bg-transparent border-2 border-surface2 text-text hover:bg-surface0 px-6 py-3 rounded-full active:scale-95 transition-all'
  };

  return (
    <button 
      className={cn(variantClasses[variant], className)}
      {...props}
    >
      {icon && <span className="inline-flex">{icon}</span>}
      {children}
    </button>
  );
};

/**
 * ModernTextField - Radius 24px, custom focus ring.
 * Matches Flutter's ModernTextField.
 */
interface TextFieldProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  prefixIcon?: React.ReactNode;
}

export const ModernTextField = ({ label, prefixIcon, className, ...props }: TextFieldProps) => (
  <div className="flex flex-col gap-2 w-full">
    {label && (
      <label className="text-xs font-bold tracking-widest text-subtext0 px-2 uppercase">
        {label}
      </label>
    )}
    <div className="relative group">
      {prefixIcon && (
        <div className="absolute left-5 top-1/2 -translate-y-1/2 text-subtext0 transition-colors group-focus-within:text-text">
          {prefixIcon}
        </div>
      )}
      <input 
        className={cn(
          "w-full bg-mantle border-2 border-transparent rounded-[24px] py-4 pr-6 text-text placeholder:text-surface2 transition-all outline-none",
          prefixIcon ? "pl-14" : "pl-6",
          "focus:border-text focus:bg-base",
          className
        )}
        {...props}
      />
    </div>
  </div>
);

/**
 * AroChip - Pill-shaped badges with alpha backgrounds.
 * Matches Flutter's AroChip.
 */
interface AroChipProps {
  label: string;
  color?: string; // Tailwind color class like 'blue', 'green', 'mauve'
  className?: string;
}

export const AroChip = ({ label, color = 'blue', className }: AroChipProps) => {
  const colorClasses: Record<string, string> = {
    blue: 'bg-blue/10 text-blue border-blue/20',
    green: 'bg-green/10 text-green border-green/20',
    mauve: 'bg-mauve/10 text-mauve border-mauve/20',
    peach: 'bg-peach/10 text-peach border-peach/20',
    yellow: 'bg-yellow/10 text-yellow border-yellow/20',
    red: 'bg-red/10 text-red border-red/20',
    sapphire: 'bg-sapphire/10 text-sapphire border-sapphire/20',
  };

  return (
    <div className={cn(
      "px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest border transition-all",
      colorClasses[color] || colorClasses.blue,
      className
    )}>
      {label}
    </div>
  );
};

/**
 * AroIconButton - Square-rounded buttons for actions.
 * Matches Flutter's AroIconButton.
 */
interface AroIconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  icon: React.ReactNode;
}

export const AroIconButton = ({ icon, className, ...props }: AroIconButtonProps) => (
  <button 
    className={cn(
      "w-12 h-12 flex items-center justify-center bg-mantle border border-surface0 rounded-2xl text-text hover:bg-surface0 active:scale-95 transition-all",
      className
    )}
    {...props}
  >
    {icon}
  </button>
);
