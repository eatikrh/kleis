import { Tabs, Tab, TabTitleText } from '@patternfly/react-core';
import { useState } from 'react';
import { paletteTabs } from './buttonConfigs';
import { PaletteButton } from './PaletteButton';
import type { EditorNode } from '../../types/ast';

interface PaletteTabsProps {
  onInsert: (ast: EditorNode) => void;
}

/**
 * Tabbed palette containing math symbol buttons
 */
export function PaletteTabs({ onInsert }: PaletteTabsProps) {
  const [activeTab, setActiveTab] = useState(paletteTabs[0]?.id || 'basic');

  return (
    <div className="palette-container">
      <Tabs
        activeKey={activeTab}
        onSelect={(_, key) => setActiveTab(key as string)}
        aria-label="Math symbol palette"
      >
        {paletteTabs.map((tab) => (
          <Tab
            key={tab.id}
            eventKey={tab.id}
            title={<TabTitleText>{tab.title}</TabTitleText>}
          >
            <div
              className="palette-buttons"
              style={{
                display: 'flex',
                flexWrap: 'wrap',
                gap: '8px',
                padding: '12px',
              }}
            >
              {tab.buttons.map((btnConfig) => (
                <PaletteButton
                  key={btnConfig.template}
                  config={btnConfig}
                  onInsert={onInsert}
                />
              ))}
            </div>
          </Tab>
        ))}
      </Tabs>
    </div>
  );
}

