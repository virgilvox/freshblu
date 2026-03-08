import type { Meta, StoryObj } from '@storybook/svelte';
import Table from './Table.svelte';

const meta = {
  title: 'UI/Table',
  component: Table,
  tags: ['autodocs'],
} satisfies Meta<Table>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    columns: ['UUID', 'Name', 'Type', 'Status'],
    children: '' as any,
  },
};
