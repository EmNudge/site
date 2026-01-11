export interface Command {
  id: string;
  title: string;
  parent?: string;
  children?: string[];
  handler?: () => void;
}
