import { Tabs, Tab, TabTitleText } from '@patternfly/react-core';
import { useState } from 'react';
import * as React from 'react';
import { paletteTabs } from './buttonConfigs';
import { PaletteButton } from './PaletteButton';
import type { EditorNode } from '../../types/ast';

interface PaletteTabsProps {
  onInsert: (ast: EditorNode) => void;
  onOpenMatrixBuilder?: () => void;
  onOpenPiecewiseBuilder?: () => void;
}

/**
 * Tabbed palette containing math symbol buttons
 */
export function PaletteTabs({ onInsert, onOpenMatrixBuilder, onOpenPiecewiseBuilder }: PaletteTabsProps) {
  const [activeTab, setActiveTab] = useState(paletteTabs[0]?.id || 'basic');

  // Expose modal openers to window for button configs
  React.useEffect(() => {
    if (onOpenMatrixBuilder) {
      (window as any).openMatrixBuilder = onOpenMatrixBuilder;
    }
    if (onOpenPiecewiseBuilder) {
      (window as any).openPiecewiseBuilder = onOpenPiecewiseBuilder;
    }
    return () => {
      delete (window as any).openMatrixBuilder;
      delete (window as any).openPiecewiseBuilder;
    };
  }, [onOpenMatrixBuilder, onOpenPiecewiseBuilder]);

  return (
    <div className="palette-container">
      <Tabs
        activeKey={activeTab}
        onSelect={(_, key) => {
          // Don't clear focus when switching tabs - preserve active marker
          setActiveTab(key as string);
        }}
        aria-label="Math symbol palette"
        mountOnEnter
        unmountOnExit={false}
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
                gap: '4px', /* Closer together */
                padding: '12px',
              }}
            >
              {tab.buttons.map((btnConfig, index) => {
                // Generate unique key combining tab ID, template name, and index
                // This ensures uniqueness even when templates repeat within the same tab
                const uniqueKey = `${tab.id}-${btnConfig.template}-${index}`;
                return (
                  <PaletteButton
                    key={uniqueKey}
                    config={btnConfig}
                    onInsert={onInsert}
                  />
                );
              })}
            </div>
          </Tab>
        ))}
      </Tabs>
    </div>
  );
}

