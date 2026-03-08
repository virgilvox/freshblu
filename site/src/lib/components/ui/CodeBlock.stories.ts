import type { Meta, StoryObj } from '@storybook/svelte';
import CodeBlock from './CodeBlock.svelte';

const meta = {
  title: 'UI/CodeBlock',
  component: CodeBlock,
  tags: ['autodocs'],
} satisfies Meta<CodeBlock>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Rust: Story = {
  args: {
    lang: 'rust',
    code: `pub struct Device {\n    pub uuid: Uuid,\n    pub name: Option<String>,\n    pub online: bool,\n}`,
  },
};

export const Bash: Story = {
  args: {
    lang: 'bash',
    code: `curl -X POST http://localhost:3000/devices \\\n  -H "Content-Type: application/json" \\\n  -d '{}'`,
  },
};

export const NoLang: Story = {
  args: { code: '{"uuid": "abc-123", "token": "xyz-789"}' },
};
