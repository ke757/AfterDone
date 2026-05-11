import { cn } from '@/lib/utils';

interface ResizeHandleProps {
  side: 'left' | 'right';
  onMouseDown: (e: React.MouseEvent) => void;
}

export function ResizeHandle({ side, onMouseDown }: ResizeHandleProps) {
  return (
    <div
      className={cn(
        'w-1 shrink-0 cursor-col-resize transition-colors hover:bg-primary/30 active:bg-primary/50',
        side === 'left' ? 'ml-0' : 'mr-0',
      )}
      onMouseDown={onMouseDown}
    />
  );
}