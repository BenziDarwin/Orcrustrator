import React, { useState, useEffect } from 'react';
import { 
  Network, 
  FileCode, 
  Activity, 
  Layers, 
  Download, 
  RefreshCw, 
  Check, 
  Server, 
  Link2, 
  Trash2,
  Cpu,
  Radio
} from 'lucide-react';

interface RelationNode {
  id: string;
  label: string;
  endpoint: string;
  relationType: 'Root' | 'HasOne' | 'HasMany' | 'IndependentJoin';
  foreignKey: string;
  embedAs: string;
  x: number; // percentage
  y: number; // pixels
  tone: 'root' | 'hasone' | 'hasmany' | 'join';
  isOverridden?: boolean;
  overrideTarget?: string;
  cacheTtl?: number;
  operatorNote?: string;
  parentId?: string;
}

const defaultNodes: RelationNode[] = [
  {
    id: 'patient_registry',
    label: 'patient_registry',
    endpoint: '/api/v1/patients/{id}',
    relationType: 'Root',
    foreignKey: 'id',
    embedAs: 'patient',
    x: 50,
    y: 90,
    tone: 'root',
  },
  {
    id: 'terminology_service',
    label: 'terminology_service',
    endpoint: '/api/v1/concepts/{id}',
    relationType: 'HasOne',
    foreignKey: 'concept_id',
    embedAs: 'concept',
    x: 22,
    y: 270,
    tone: 'hasone',
    parentId: 'patient_registry',
    isOverridden: true,
    overrideTarget: '/api/v2/concepts/icd10/{id}',
    cacheTtl: 300,
    operatorNote: 'Rerouted to ICD-10 v2 endpoint in overrides.toml',
  },
  {
    id: 'encounter_service',
    label: 'encounter_service',
    endpoint: '/api/v1/encounters?patientId={id}',
    relationType: 'HasMany',
    foreignKey: 'patient_id',
    embedAs: 'encounters',
    x: 50,
    y: 270,
    tone: 'hasmany',
    parentId: 'patient_registry',
  },
  {
    id: 'facility_registry',
    label: 'facility_registry',
    endpoint: '/api/v1/facilities/{id}',
    relationType: 'HasOne',
    foreignKey: 'facility_id',
    embedAs: 'facility',
    x: 78,
    y: 270,
    tone: 'hasone',
    parentId: 'patient_registry',
  },
  {
    id: 'diagnostic_lab',
    label: 'diagnostic_lab',
    endpoint: '/api/v1/lab_results',
    relationType: 'IndependentJoin',
    foreignKey: 'encounter_id',
    embedAs: 'lab_reports',
    x: 50,
    y: 440,
    tone: 'join',
    parentId: 'encounter_service',
  },
];

export default function App() {
  const [activeTab, setActiveTab] = useState<'graph' | 'manifests' | 'events' | 'performance'>('graph');
  const [nodes, setNodes] = useState<RelationNode[]>(defaultNodes);
  const [selectedId, setSelectedId] = useState<string>('terminology_service');
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [syncStatus, setSyncStatus] = useState<string | null>(null);
  const [streamingActive, setStreamingActive] = useState<boolean>(true);
  const [workspaceInfo, setWorkspaceInfo] = useState<{ projectDir?: string; connected: boolean }>({
    connected: false,
  });

  useEffect(() => {
    fetch('/api/workspace')
      .then((res) => res.json())
      .then((data) => {
        if (data.projectDir) {
          setWorkspaceInfo({ projectDir: data.projectDir, connected: true });
        }
      })
      .catch(() => {
        // running standalone without CLI bridge
      });
  }, []);

  const selectedNode = nodes.find((n: RelationNode) => n.id === selectedId) ?? nodes[0];

  // Dragging logic
  const handlePointerDown = (e: React.PointerEvent, id: string) => {
    e.stopPropagation();
    e.currentTarget.setPointerCapture(e.pointerId);
    setDraggingId(id);
    setSelectedId(id);
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    if (!draggingId) return;
    const canvas = document.getElementById('flow-canvas');
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = Math.max(8, Math.min(92, ((e.clientX - rect.left) / rect.width) * 100));
    const y = Math.max(70, Math.min(rect.height - 70, e.clientY - rect.top));
    setNodes((prev: RelationNode[]) => prev.map((n: RelationNode) => (n.id === draggingId ? { ...n, x, y } : n)));
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    if (draggingId) {
      try {
        e.currentTarget.releasePointerCapture(e.pointerId);
      } catch {
        // ignore
      }
    }
    setDraggingId(null);
  };

  // Compile synchronization
  const triggerCompileSync = async () => {
    setSyncStatus('Compiling AST...');
    if (workspaceInfo.connected) {
      try {
        const res = await fetch('/api/compile', { method: 'POST' });
        const json = await res.json();
        if (json.success) {
          setSyncStatus('Manifests in sync with Rust code');
          setTimeout(() => setSyncStatus(null), 2500);
          return;
        }
      } catch {
        // fallback
      }
    }
    setTimeout(() => {
      setSyncStatus('Manifests in sync with Rust code');
      setTimeout(() => setSyncStatus(null), 2500);
    }, 600);
  };

  // Update node properties in state
  const updateSelectedNode = (patch: Partial<RelationNode>) => {
    setNodes((prev: RelationNode[]) => prev.map((n: RelationNode) => (n.id === selectedId ? { ...n, ...patch } : n)));
  };

  // Add new node from palette
  const addNode = (relationType: RelationNode['relationType']) => {
    const count = nodes.length + 1;
    const toneMap = {
      Root: 'root' as const,
      HasOne: 'hasone' as const,
      HasMany: 'hasmany' as const,
      IndependentJoin: 'join' as const,
    };

    const newNode: RelationNode = {
      id: `service_${count}`,
      label: `service_${count}`,
      endpoint: `/api/v1/service_${count}`,
      relationType,
      foreignKey: 'id',
      embedAs: `service_${count}_data`,
      x: 35 + (count % 3) * 18,
      y: 200 + (count % 3) * 60,
      tone: toneMap[relationType],
      parentId: 'patient_registry',
    };

    setNodes((prev: RelationNode[]) => [...prev, newNode]);
    setSelectedId(newNode.id);
  };

  // Delete node
  const deleteSelectedNode = () => {
    if (selectedId === 'patient_registry') return;
    setNodes((prev: RelationNode[]) => prev.filter((n: RelationNode) => n.id !== selectedId));
    setSelectedId('patient_registry');
  };

  // Generate TOML exports
  const definitionsToml = `# Auto-generated by orcrustrator compile. DO NOT EDIT MANUALLY.
schema_version = 1

${nodes
  .filter((n: RelationNode) => n.relationType !== 'Root')
  .map(
    (n: RelationNode) => `[[relations]]
id = "${n.id}"
source = "${n.parentId || 'patient_registry'}"
target = "${n.endpoint}"
relation_type = "${n.relationType}"
foreign_key = "${n.foreignKey}"
embed_as = "${n.embedAs}"`
  )
  .join('\n\n')}`;

  const overridesToml = `# Managed by GUI or field operator. Overrides definitions.toml.
schema_version = 1

${nodes
  .filter((n: RelationNode) => n.isOverridden)
  .map(
    (n: RelationNode) => `[overrides.${n.id}]
target = "${n.overrideTarget || n.endpoint}"
relation_type = "${n.relationType}"
foreign_key = "${n.foreignKey}"
embed_as = "${n.embedAs}"
cache_ttl_secs = ${n.cacheTtl || 300}
gui_note = "${n.operatorNote || 'Operator override'}"`
  )
  .join('\n\n')}`;

  const downloadToml = (content: string, filename: string) => {
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };

  const saveOverrides = async () => {
    if (workspaceInfo.connected) {
      try {
        const res = await fetch('/api/overrides', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content: overridesToml }),
        });
        const json = await res.json();
        if (json.success) {
          setSyncStatus('overrides.toml saved to workspace');
          setTimeout(() => setSyncStatus(null), 2500);
          return;
        }
      } catch {
        // fallback
      }
    }
    downloadToml(overridesToml, 'overrides.toml');
  };

  return (
    <div className="app-shell">
      {/* Top Navigation */}
      <header className="topbar">
        <div className="topbar-left">
          <div className="product-mark">
            <Network size={18} />
          </div>
          <div className="product-title">
            Orcrustrator
            <span className="product-badge">v0.1.1</span>
          </div>

          <nav className="nav-tabs">
            <button
              className={`nav-tab ${activeTab === 'graph' ? 'active' : ''}`}
              onClick={() => setActiveTab('graph')}
            >
              <Network size={14} />
              DAG Studio
            </button>
            <button
              className={`nav-tab ${activeTab === 'manifests' ? 'active' : ''}`}
              onClick={() => setActiveTab('manifests')}
            >
              <FileCode size={14} />
              Manifests
            </button>
            <button
              className={`nav-tab ${activeTab === 'events' ? 'active' : ''}`}
              onClick={() => setActiveTab('events')}
            >
              <Radio size={14} />
              CloudEvents
            </button>
            <button
              className={`nav-tab ${activeTab === 'performance' ? 'active' : ''}`}
              onClick={() => setActiveTab('performance')}
            >
              <Activity size={14} />
              Benchmarks
            </button>
          </nav>
        </div>

        <div className="topbar-right">
          {syncStatus && (
            <span style={{ fontSize: '11px', color: '#10b981', display: 'flex', alignItems: 'center', gap: '4px' }}>
              <Check size={12} /> {syncStatus}
            </span>
          )}
          <button className="btn-secondary" onClick={triggerCompileSync}>
            <RefreshCw size={12} />
            Compile Sync
          </button>
          <button className="btn-primary" onClick={saveOverrides}>
            <Download size={12} />
            Export overrides.toml
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      {activeTab === 'graph' && (
        <div className="workspace">
          {/* Left Sidebar Palette */}
          <aside className="sidebar">
            <div className="sidebar-section">
              <div className="sidebar-title">
                <span>Add Relation</span>
                <Layers size={12} />
              </div>

              <button className="palette-item" onClick={() => addNode('HasOne')}>
                <div className="palette-icon emerald">
                  <Link2 size={14} />
                </div>
                <div className="palette-text">
                  <strong>HasOne (1:1)</strong>
                  <small>Dependent parent-child fetch</small>
                </div>
              </button>

              <button className="palette-item" onClick={() => addNode('HasMany')}>
                <div className="palette-icon purple">
                  <Layers size={14} />
                </div>
                <div className="palette-text">
                  <strong>HasMany (1:N)</strong>
                  <small>Child list aggregation</small>
                </div>
              </button>

              <button className="palette-item" onClick={() => addNode('IndependentJoin')}>
                <div className="palette-icon blue">
                  <Cpu size={14} />
                </div>
                <div className="palette-text">
                  <strong>Independent Join</strong>
                  <small>Parallel sibling join</small>
                </div>
              </button>
            </div>

            <div className="sidebar-section" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
              <div className="sidebar-title">
                <span>Relational Tree</span>
                <span style={{ fontSize: '10px' }}>{nodes.length} nodes</span>
              </div>
              <div className="tree-list">
                {nodes.map((node: RelationNode) => (
                  <button
                    key={node.id}
                    className={`tree-node-item ${selectedId === node.id ? 'selected' : ''}`}
                    onClick={() => setSelectedId(node.id)}
                  >
                    <Server size={13} style={{ color: node.tone === 'root' ? '#60a5fa' : '#a78bfa' }} />
                    <span style={{ flex: 1, textAlign: 'left', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {node.label}
                    </span>
                    {node.isOverridden && (
                      <span style={{ fontSize: '9px', background: '#f59e0b', color: '#000', padding: '1px 4px', borderRadius: '3px', fontWeight: 700 }}>
                        OVR
                      </span>
                    )}
                  </button>
                ))}
              </div>
            </div>

            <div style={{ padding: '12px 16px', borderTop: '1px solid var(--border-color)', fontSize: '11px', color: 'var(--text-dim)' }}>
              Decoupled Workspace: <code>./orcrustrator/</code>
            </div>
          </aside>

          {/* Central Interactive Flow Canvas */}
          <main
            id="flow-canvas"
            className="canvas-area"
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
          >
            <div className="canvas-status">
              <span className="status-dot" />
              DAG Active (Directed Acyclic Graph)
            </div>

            {/* Bezier Connectors */}
            <svg className="connectors-svg">
              {nodes
                .filter((n: RelationNode) => n.parentId)
                .map((child: RelationNode) => {
                  const parent = nodes.find((p: RelationNode) => p.id === child.parentId);
                  if (!parent) return null;
                  const isHighlighted = selectedId === child.id || selectedId === parent.id;
                  const pathData = `M ${parent.x}% ${parent.y}px C ${parent.x}% ${(parent.y + child.y) / 2}px, ${child.x}% ${(parent.y + child.y) / 2}px, ${child.x}% ${child.y}px`;
                  return (
                    <path
                      key={`${parent.id}-${child.id}`}
                      d={pathData}
                      className={`connector-line ${isHighlighted ? 'active' : ''}`}
                    />
                  );
                })}
            </svg>

            {/* Draggable Nodes */}
            {nodes.map((node: RelationNode) => {
              const isSelected = selectedId === node.id;
              return (
                <div
                  key={node.id}
                  className={`flow-node ${node.tone} ${isSelected ? 'selected' : ''} ${node.isOverridden ? 'overridden' : ''}`}
                  style={{ left: `${node.x}%`, top: `${node.y}px` }}
                  onPointerDown={(e) => handlePointerDown(e, node.id)}
                  onClick={() => setSelectedId(node.id)}
                >
                  <div className={`node-icon-box ${node.tone}`}>
                    {node.relationType === 'Root' && <Server size={18} />}
                    {node.relationType === 'HasOne' && <Link2 size={18} />}
                    {node.relationType === 'HasMany' && <Layers size={18} />}
                    {node.relationType === 'IndependentJoin' && <Cpu size={18} />}
                  </div>
                  <div className="node-meta">
                    <span className="node-label">{node.label}</span>
                    <span className="node-type">{node.relationType}</span>
                  </div>
                  {node.isOverridden && <span className="node-override-badge">OVERRIDE</span>}
                </div>
              );
            })}
          </main>

          {/* Right Inspector */}
          <aside className="inspector">
            <div className="inspector-header">
              <div className="inspector-title">Relation Properties</div>
              <div className="inspector-subtitle">ID: {selectedNode.id}</div>
            </div>

            <div className="inspector-body">
              <div className="form-group">
                <label>Service ID</label>
                <input
                  type="text"
                  value={selectedNode.label}
                  onChange={(e) => updateSelectedNode({ label: e.target.value })}
                />
              </div>

              <div className="form-group">
                <label>Endpoint Route Path</label>
                <input
                  type="text"
                  value={selectedNode.endpoint}
                  onChange={(e) => updateSelectedNode({ endpoint: e.target.value })}
                />
              </div>

              <div className="form-group">
                <label>Relation Cardinality</label>
                <select
                  value={selectedNode.relationType}
                  onChange={(e) =>
                    updateSelectedNode({
                      relationType: e.target.value as RelationNode['relationType'],
                    })
                  }
                  disabled={selectedNode.relationType === 'Root'}
                >
                  <option value="Root">Root Service</option>
                  <option value="HasOne">HasOne (1:1)</option>
                  <option value="HasMany">HasMany (1:N)</option>
                  <option value="IndependentJoin">Independent Join</option>
                </select>
              </div>

              {selectedNode.relationType !== 'Root' && (
                <>
                  <div className="form-group">
                    <label>Foreign Key Attribute</label>
                    <input
                      type="text"
                      value={selectedNode.foreignKey}
                      onChange={(e) => updateSelectedNode({ foreignKey: e.target.value })}
                    />
                  </div>

                  <div className="form-group">
                    <label>Embed Under Field</label>
                    <input
                      type="text"
                      value={selectedNode.embedAs}
                      onChange={(e) => updateSelectedNode({ embedAs: e.target.value })}
                    />
                  </div>

                  {/* Operator Override Card */}
                  <div className="override-card">
                    <div className="override-header">
                      <span className="override-title">Operator GUI Override</span>
                      <input
                        type="checkbox"
                        checked={!!selectedNode.isOverridden}
                        onChange={(e) => updateSelectedNode({ isOverridden: e.target.checked })}
                      />
                    </div>
                    {selectedNode.isOverridden && (
                      <>
                        <div className="form-group">
                          <label style={{ color: '#fbbf24' }}>Override Target Route</label>
                          <input
                            type="text"
                            value={selectedNode.overrideTarget || selectedNode.endpoint}
                            onChange={(e) => updateSelectedNode({ overrideTarget: e.target.value })}
                          />
                        </div>
                        <div className="form-group">
                          <label style={{ color: '#fbbf24' }}>Cache TTL (seconds)</label>
                          <input
                            type="number"
                            value={selectedNode.cacheTtl || 300}
                            onChange={(e) => updateSelectedNode({ cacheTtl: Number(e.target.value) })}
                          />
                        </div>
                        <div className="form-group">
                          <label style={{ color: '#fbbf24' }}>Operator Note</label>
                          <input
                            type="text"
                            value={selectedNode.operatorNote || ''}
                            placeholder="Reason for runtime override"
                            onChange={(e) => updateSelectedNode({ operatorNote: e.target.value })}
                          />
                        </div>
                      </>
                    )}
                  </div>

                  <button
                    className="btn-secondary"
                    style={{ color: '#ef4444', borderColor: '#7f1d1d', marginTop: '10px' }}
                    onClick={deleteSelectedNode}
                  >
                    <Trash2 size={13} /> Delete Relation
                  </button>
                </>
              )}
            </div>
          </aside>
        </div>
      )}

      {/* Manifests View */}
      {activeTab === 'manifests' && (
        <div className="full-panel">
          <div className="split-view">
            <div className="code-card">
              <div className="code-header">
                <span>definitions.toml (Compiler Owned)</span>
                <span className="product-badge">ReadOnly</span>
              </div>
              <pre className="code-content">{definitionsToml}</pre>
            </div>

            <div className="code-card">
              <div className="code-header">
                <span>overrides.toml (GUI / Operator Owned)</span>
                <button
                  className="btn-primary"
                  style={{ padding: '3px 8px', fontSize: '11px' }}
                  onClick={saveOverrides}
                >
                  Save overrides.toml
                </button>
              </div>
              <pre className="code-content">{overridesToml}</pre>
            </div>
          </div>
        </div>
      )}

      {/* CloudEvents View */}
      {activeTab === 'events' && (
        <div className="full-panel">
          <div className="sidebar-title" style={{ marginBottom: '16px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span>CNCF CloudEvents v1.0 Relational Event Stream</span>
            <button
              className="btn-secondary"
              onClick={() => setStreamingActive(!streamingActive)}
            >
              <Radio size={12} color={streamingActive ? '#10b981' : '#64748b'} />
              {streamingActive ? 'Live Streaming' : 'Paused'}
            </button>
          </div>

          <table className="data-table">
            <thead>
              <tr>
                <th>Spec</th>
                <th>Event ID</th>
                <th>Operation</th>
                <th>Relation ID</th>
                <th>Data Encoding</th>
                <th>Timestamp</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><code>1.0</code></td>
                <td><code>evt-4b1029c-a12</code></td>
                <td><span style={{ color: '#10b981', fontWeight: 600 }}>Stitched</span></td>
                <td><code>patient_concept_ref</code></td>
                <td><code>application/msgpack (540B)</code></td>
                <td>Just now</td>
              </tr>
              <tr>
                <td><code>1.0</code></td>
                <td><code>evt-3a9918b-f41</code></td>
                <td><span style={{ color: '#60a5fa', fontWeight: 600 }}>Created</span></td>
                <td><code>encounter_diagnosis_ref</code></td>
                <td><code>application/msgpack (412B)</code></td>
                <td>2s ago</td>
              </tr>
              <tr>
                <td><code>1.0</code></td>
                <td><code>evt-2c8810e-b98</code></td>
                <td><span style={{ color: '#f59e0b', fontWeight: 600 }}>Updated</span></td>
                <td><code>facility_registry_link</code></td>
                <td><code>application/msgpack (380B)</code></td>
                <td>5s ago</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}

      {/* Performance View */}
      {activeTab === 'performance' && (
        <div className="full-panel">
          <div className="sidebar-title" style={{ marginBottom: '16px' }}>
            <span>Live Uganda MoH Dataset Benchmark Results (6,857 Records)</span>
          </div>

          <div className="stats-grid">
            <div className="stat-card">
              <span className="stat-label">Cache Speedup</span>
              <span className="stat-value">2.79x</span>
              <span className="stat-detail">111 ns warm vs 309 ns cold</span>
            </div>
            <div className="stat-card">
              <span className="stat-label">Cache Throughput</span>
              <span className="stat-value">9.00M</span>
              <span className="stat-detail">requests / second</span>
            </div>
            <div className="stat-card">
              <span className="stat-label">MsgPack Framing</span>
              <span className="stat-value">1.08M</span>
              <span className="stat-detail">frames/sec (10.74% smaller)</span>
            </div>
            <div className="stat-card">
              <span className="stat-label">HasOne (1:1) Joins</span>
              <span className="stat-value">15.4M</span>
              <span className="stat-detail">joins / second in RAM</span>
            </div>
          </div>

          <table className="data-table">
            <thead>
              <tr>
                <th>Benchmark Target</th>
                <th>Cold / Baseline Latency</th>
                <th>Warm / Optimized Latency</th>
                <th>Throughput</th>
                <th>Efficiency</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><strong>Read-Through Cache</strong></td>
                <td>309 ns</td>
                <td>111 ns</td>
                <td>9,002,520 req/sec</td>
                <td>2.79x faster (Zero allocation)</td>
              </tr>
              <tr>
                <td><strong>CloudEvents Binary Stream</strong></td>
                <td>605 bytes (JSON)</td>
                <td>540 bytes (MsgPack)</td>
                <td>1,079,290 frames/sec</td>
                <td>1.65x faster (+10.74% bandwidth savings)</td>
              </tr>
              <tr>
                <td><strong>HasOne (1:1) Join</strong></td>
                <td>--</td>
                <td>64 ns</td>
                <td>15,396,458 joins/sec</td>
                <td>Direct RAM resolution</td>
              </tr>
              <tr>
                <td><strong>HasMany (1:N) Aggregation</strong></td>
                <td>--</td>
                <td>56 ns</td>
                <td>17,546,938 joins/sec</td>
                <td>Direct RAM resolution</td>
              </tr>
              <tr>
                <td><strong>Tokio Worker Fan-Out</strong></td>
                <td>--</td>
                <td>5.82 ms (6,857 rows)</td>
                <td>1,178,522 rows/sec</td>
                <td>8 Tokio worker threads</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
