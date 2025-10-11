// SemVerX Interactive System
class SemVerXSystem {
    constructor() {
        this.components = new Map();
        this.dependencies = new Map();
        this.currentLayer = null;
        this.init();
    }

    init() {
        this.setupAnimations();
        this.setupCharts();
        this.setupParticleSystem();
        this.setupTypedText();
    }

    setupAnimations() {
        // Animate architecture layers on scroll
        const layers = document.querySelectorAll('.architecture-layer');
        
        const observer = new IntersectionObserver((entries) => {
            entries.forEach((entry, index) => {
                if (entry.isIntersecting) {
                    anime({
                        targets: entry.target,
                        opacity: [0, 1],
                        translateY: [30, 0],
                        delay: index * 200,
                        duration: 800,
                        easing: 'easeOutCubic'
                    });
                }
            });
        }, { threshold: 0.1 });

        layers.forEach(layer => observer.observe(layer));

        // Animate component cards on hover
        document.querySelectorAll('.component-card').forEach(card => {
            card.addEventListener('mouseenter', () => {
                anime({
                    targets: card,
                    scale: 1.02,
                    rotateX: 5,
                    duration: 300,
                    easing: 'easeOutCubic'
                });
            });

            card.addEventListener('mouseleave', () => {
                anime({
                    targets: card,
                    scale: 1,
                    rotateX: 0,
                    duration: 300,
                    easing: 'easeOutCubic'
                });
            });
        });
    }

    setupCharts() {
        // Architecture Flow Chart
        const archChart = echarts.init(document.getElementById('architecture-chart'));
        
        const archOption = {
            backgroundColor: 'transparent',
            tooltip: {
                trigger: 'item',
                backgroundColor: 'rgba(11, 12, 16, 0.9)',
                borderColor: '#66FCF1',
                textStyle: { color: '#FFFFFF' }
            },
            series: [{
                type: 'graph',
                layout: 'force',
                animation: true,
                roam: true,
                focusNodeAdjacency: true,
                force: {
                    repulsion: 1000,
                    gravity: 0.1,
                    edgeLength: 200
                },
                data: [
                    { name: 'CDIS', value: 100, category: 0, symbolSize: 60 },
                    { name: 'HDIS', value: 100, category: 1, symbolSize: 60 },
                    { name: 'QDIS', value: 100, category: 2, symbolSize: 60 },
                    { name: 'SemVerX Core', value: 100, category: 3, symbolSize: 80 },
                    { name: 'Dependency Engine', value: 100, category: 4, symbolSize: 50 },
                    { name: 'Polyglot Interface', value: 100, category: 5, symbolSize: 50 }
                ],
                links: [
                    { source: 'CDIS', target: 'HDIS' },
                    { source: 'HDIS', target: 'QDIS' },
                    { source: 'QDIS', target: 'HDIS' },
                    { source: 'HDIS', target: 'SemVerX Core' },
                    { source: 'SemVerX Core', target: 'CDIS' },
                    { source: 'SemVerX Core', target: 'Dependency Engine' },
                    { source: 'Dependency Engine', target: 'SemVerX Core' },
                    { source: 'QDIS', target: 'Dependency Engine' },
                    { source: 'SemVerX Core', target: 'Polyglot Interface' },
                    { source: 'Polyglot Interface', target: 'SemVerX Core' }
                ],
                categories: [
                    { name: 'CDIS', itemStyle: { color: '#45A29E' } },
                    { name: 'HDIS', itemStyle: { color: '#66FCF1' } },
                    { name: 'QDIS', itemStyle: { color: '#C5C6C7' } },
                    { name: 'SemVerX Core', itemStyle: { color: '#FFB347' } },
                    { name: 'Dependency Engine', itemStyle: { color: '#FF6B6B' } },
                    { name: 'Polyglot Interface', itemStyle: { color: '#8A2BE2' } }
                ],
                lineStyle: {
                    color: '#FFFFFF',
                    width: 2,
                    curveness: 0.1,
                    opacity: 0.8
                },
                label: {
                    show: true,
                    position: 'bottom',
                    color: '#FFFFFF',
                    fontSize: 12
                },
                emphasis: {
                    lineStyle: {
                        width: 4,
                        opacity: 1
                    }
                }
            }]
        };

        archChart.setOption(archOption);

        // Dependency Graph
        this.dependencyChart = echarts.init(document.getElementById('dependency-graph'));
        this.updateDependencyGraph();

        // Handle window resize
        window.addEventListener('resize', () => {
            archChart.resize();
            this.dependencyChart.resize();
        });
    }

    setupParticleSystem() {
        // Create particle system using p5.js
        new p5((p) => {
            let particles = [];
            
            p.setup = () => {
                const canvas = p.createCanvas(p.windowWidth, p.windowHeight);
                canvas.id('particles-canvas');
                canvas.position(0, 0);
                canvas.style('z-index', '-1');
                canvas.style('position', 'fixed');
                
                // Create initial particles
                for (let i = 0; i < 50; i++) {
                    particles.push(new Particle(p));
                }
            };
            
            p.draw = () => {
                p.clear();
                
                // Update and draw particles
                particles.forEach(particle => {
                    particle.update();
                    particle.draw();
                });
                
                // Remove dead particles and add new ones
                particles = particles.filter(p => p.isAlive());
                while (particles.length < 50) {
                    particles.push(new Particle(p));
                }
            };
            
            p.windowResized = () => {
                p.resizeCanvas(p.windowWidth, p.windowHeight);
            };
            
            class Particle {
                constructor(p) {
                    this.p = p;
                    this.x = p.random(p.width);
                    this.y = p.random(p.height);
                    this.vx = p.random(-0.5, 0.5);
                    this.vy = p.random(-0.5, 0.5);
                    this.life = 255;
                    this.decay = p.random(0.5, 2);
                }
                
                update() {
                    this.x += this.vx;
                    this.y += this.vy;
                    this.life -= this.decay;
                    
                    // Wrap around edges
                    if (this.x < 0) this.x = this.p.width;
                    if (this.x > this.p.width) this.x = 0;
                    if (this.y < 0) this.y = this.p.height;
                    if (this.y > this.p.height) this.y = 0;
                }
                
                draw() {
                    this.p.fill(102, 252, 241, this.life * 0.3);
                    this.p.noStroke();
                    this.p.circle(this.x, this.y, 2);
                }
                
                isAlive() {
                    return this.life > 0;
                }
            }
        });
    }

    setupTypedText() {
        new Typed('#typed-text', {
            strings: [
                'Semantic Version Extended',
                'Hot-Swappable Components',
                'Graph-Based Dependency Resolution',
                'Polyglot Architecture Support',
                'Zero-Downtime Updates'
            ],
            typeSpeed: 50,
            backSpeed: 30,
            backDelay: 2000,
            loop: true,
            showCursor: true,
            cursorChar: '_'
        });
    }

    updateDependencyGraph() {
        const nodes = Array.from(this.components.keys()).map(name => ({
            name,
            value: 50,
            symbolSize: 40,
            category: this.getComponentState(name)
        }));

        const links = [];
        this.dependencies.forEach((deps, source) => {
            deps.forEach(target => {
                links.push({ source, target });
            });
        });

        const option = {
            backgroundColor: 'transparent',
            tooltip: {
                trigger: 'item',
                backgroundColor: 'rgba(11, 12, 16, 0.9)',
                borderColor: '#66FCF1',
                textStyle: { color: '#FFFFFF' }
            },
            series: [{
                type: 'graph',
                layout: 'force',
                animation: true,
                roam: true,
                data: nodes,
                links: links,
                categories: [
                    { name: 'stable', itemStyle: { color: '#45A29E' } },
                    { name: 'experimental', itemStyle: { color: '#FFB347' } },
                    { name: 'legacy', itemStyle: { color: '#FF6B6B' } }
                ],
                force: {
                    repulsion: 500,
                    gravity: 0.1,
                    edgeLength: 100
                },
                lineStyle: {
                    color: '#FFFFFF',
                    width: 2,
                    opacity: 0.6
                },
                label: {
                    show: true,
                    position: 'bottom',
                    color: '#FFFFFF',
                    fontSize: 10
                }
            }]
        };

        this.dependencyChart.setOption(option);
    }

    getComponentState(name) {
        const component = this.components.get(name);
        return component ? component.state : 'stable';
    }
}

// Global system instance
const semverxSystem = new SemVerXSystem();

// Interactive Functions
function exploreArchitecture() {
    document.getElementById('architecture').scrollIntoView({ behavior: 'smooth' });
}

function viewDemo() {
    document.getElementById('demo').scrollIntoView({ behavior: 'smooth' });
}

function exploreLayer(layer) {
    // Animate layer selection
    const layers = document.querySelectorAll('.architecture-layer');
    layers.forEach(l => {
        if (l.onclick.toString().includes(layer)) {
            anime({
                targets: l,
                scale: [1, 1.05, 1],
                duration: 600,
                easing: 'easeOutElastic(1, .8)'
            });
        }
    });

    // Show layer information
    showLayerInfo(layer);
}

function showLayerInfo(layer) {
    const info = {
        cdis: {
            title: 'Classical Deterministic Instruction System',
            description: 'Provides the foundational layer of fixed logic and quality assurance rules. Ensures deterministic behavior and validates version schemas.',
            features: ['Static Rule Engine', 'Version Schema Parser', 'QA Matrix Validator']
        },
        hdis: {
            title: 'Hybrid Directed Instruction System', 
            description: 'Adaptive engine that manages component evolution and resolves dependency intents using advanced pathfinding algorithms.',
            features: ['Evolution Engine', 'Intent Resolver', 'A* Path Evaluator']
        },
        qdis: {
            title: 'Quantum Directed Instruction System',
            description: 'Exploratory layer for advanced search and optimization using graph theory concepts like Hamiltonian cycles.',
            features: ['Exploratory Node Search', 'Hamiltonian Cycle Mapper', 'Dynamic Fault Recovery']
        },
        semverx: {
            title: 'Semantic Version X Core',
            description: 'Central coordination system managing version lifecycles, hot-swapping operations, and dependency resolution.',
            features: ['Version State Machine', 'Hot Swap Controller', 'Dependency DAG Resolver']
        }
    };

    const layerInfo = info[layer];
    if (layerInfo) {
        // Create or update info panel
        let infoPanel = document.getElementById('layer-info');
        if (!infoPanel) {
            infoPanel = document.createElement('div');
            infoPanel.id = 'layer-info';
            infoPanel.className = 'fixed top-20 right-4 w-80 component-card p-6 rounded-2xl z-40';
            document.body.appendChild(infoPanel);
        }

        infoPanel.innerHTML = `
            <h3 class="font-space font-bold text-lg mb-3">${layerInfo.title}</h3>
            <p class="text-sm text-gray-300 mb-4">${layerInfo.description}</p>
            <div class="space-y-2">
                ${layerInfo.features.map(feature => `
                    <div class="flex items-center text-xs">
                        <span class="w-2 h-2 rounded-full mr-2 bg-cyan-400"></span>
                        ${feature}
                    </div>
                `).join('')}
            </div>
            <button onclick="closeLayerInfo()" class="mt-4 text-xs text-gray-400 hover:text-white">Close</button>
        `;

        anime({
            targets: infoPanel,
            opacity: [0, 1],
            translateX: [100, 0],
            duration: 300,
            easing: 'easeOutCubic'
        });
    }
}

function closeLayerInfo() {
    const infoPanel = document.getElementById('layer-info');
    if (infoPanel) {
        anime({
            targets: infoPanel,
            opacity: [1, 0],
            translateX: [0, 100],
            duration: 300,
            easing: 'easeOutCubic',
            complete: () => infoPanel.remove()
        });
    }
}

// Dependency Graph Functions
function addNode() {
    const componentNames = ['auth-service', 'database', 'api-gateway', 'cache', 'logger', 'monitor'];
    const name = componentNames[Math.floor(Math.random() * componentNames.length)] + '-' + Date.now();
    
    semverxSystem.components.set(name, {
        name,
        state: 'stable',
        version: `${Math.floor(Math.random() * 3)}.${Math.floor(Math.random() * 10)}.${Math.floor(Math.random() * 10)}`
    });

    // Add random dependencies
    const existingComponents = Array.from(semverxSystem.components.keys());
    if (existingComponents.length > 1) {
        const numDeps = Math.floor(Math.random() * 3) + 1;
        for (let i = 0; i < numDeps; i++) {
            const target = existingComponents[Math.floor(Math.random() * existingComponents.length)];
            if (target !== name) {
                if (!semverxSystem.dependencies.has(name)) {
                    semverxSystem.dependencies.set(name, new Set());
                }
                semverxSystem.dependencies.get(name).add(target);
            }
        }
    }

    semverxSystem.updateDependencyGraph();
    
    // Animate addition
    anime({
        targets: '#dependency-graph',
        scale: [0.95, 1],
        duration: 300,
        easing: 'easeOutCubic'
    });
}

function resolveDependencies() {
    // Simulate dependency resolution with animation
    anime({
        targets: '#dependency-graph',
        rotate: [0, 360],
        duration: 1000,
        easing: 'easeOutCubic'
    });

    // Show resolution status
    const status = document.createElement('div');
    status.className = 'fixed top-20 left-4 bg-green-600 text-white px-4 py-2 rounded-lg z-40';
    status.textContent = 'Dependencies resolved successfully!';
    document.body.appendChild(status);

    setTimeout(() => {
        anime({
            targets: status,
            opacity: [1, 0],
            translateY: [0, -20],
            duration: 300,
            complete: () => status.remove()
        });
    }, 2000);
}

function clearGraph() {
    semverxSystem.components.clear();
    semverxSystem.dependencies.clear();
    semverxSystem.updateDependencyGraph();
}

// Version Lifecycle Functions
function transitionVersion(newState) {
    updateLifecycleStatus(`component-a transitioned to ${newState} state`);
    animateStateTransition('component-a', newState);
}

function transitionVersion2(newState) {
    updateLifecycleStatus(`component-b transitioned to ${newState} state`);
    animateStateTransition('component-b', newState);
}

function updateLifecycleStatus(message) {
    const status = document.getElementById('lifecycle-status');
    status.textContent = message;
    
    anime({
        targets: status,
        opacity: [0.5, 1, 0.5],
        duration: 1000,
        easing: 'easeInOutSine'
    });
}

function animateStateTransition(component, state) {
    const colors = {
        stable: '#45A29E',
        experimental: '#FFB347', 
        legacy: '#FF6B6B'
    };

    // Find component element and animate color change
    const components = document.querySelectorAll('.component-card');
    components.forEach(card => {
        if (card.textContent.includes(component)) {
            anime({
                targets: card,
                backgroundColor: colors[state] + '33', // Add transparency
                duration: 500,
                easing: 'easeOutCubic'
            });
        }
    });
}

// Code Example Functions
function runCodeExample() {
    const codeElement = document.getElementById('rust-code');
    
    // Animate code execution
    anime({
        targets: codeElement,
        backgroundColor: ['#1f2937', '#065f46', '#1f2937'],
        duration: 2000,
        easing: 'easeInOutSine'
    });

    // Show execution result
    const result = document.createElement('div');
    result.className = 'mt-4 p-4 bg-green-900 rounded-lg text-green-300 font-mono text-sm';
    result.innerHTML = `
        <div>✓ Component component-a v1.2.3 added to system</div>
        <div>✓ Component component-b v2.0.1 added to system</div>
        <div>✓ Dependency relationship established</div>
        <div>✓ Hot-swap completed: component-b v2.0.1 → v2.1.0</div>
        <div>✓ Dependencies resolved using Eulerian cycle algorithm</div>
        <div class="mt-2 text-green-200">Resolved dependencies: ["component-a", "component-b"]</div>
    `;
    
    codeElement.parentNode.insertBefore(result, codeElement.nextSibling);

    // Remove result after delay
    setTimeout(() => {
        anime({
            targets: result,
            opacity: [1, 0],
            height: [result.offsetHeight, 0],
            duration: 500,
            complete: () => result.remove()
        });
    }, 4000);
}

// Smooth scrolling for navigation
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const target = document.querySelector(this.getAttribute('href'));
        if (target) {
            target.scrollIntoView({
                behavior: 'smooth',
                block: 'start'
            });
        }
    });
});

// Add scroll-based animations
window.addEventListener('scroll', () => {
    const scrolled = window.pageYOffset;
    const parallax = document.querySelector('.hero-bg');
    if (parallax) {
        const speed = scrolled * 0.5;
        parallax.style.transform = `translateY(${speed}px)`;
    }
});

// Initialize tooltips and enhanced interactions
document.addEventListener('DOMContentLoaded', () => {
    // Add loading animation
    const loader = document.createElement('div');
    loader.className = 'fixed inset-0 bg-black z-50 flex items-center justify-center';
    loader.innerHTML = `
        <div class="text-center">
            <div class="animate-spin rounded-full h-16 w-16 border-t-2 border-cyan-400 mx-auto mb-4"></div>
            <div class="text-white font-space">Loading SemVerX...</div>
        </div>
    `;
    document.body.appendChild(loader);

    // Remove loader after page loads
    window.addEventListener('load', () => {
        anime({
            targets: loader,
            opacity: [1, 0],
            duration: 500,
            complete: () => loader.remove()
        });
    });
});
