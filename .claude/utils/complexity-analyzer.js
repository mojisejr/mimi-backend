/**
 * Complexity Analysis Engine for Task Planning
 * Analyzes feature descriptions to determine implementation complexity and suggest appropriate workflow
 */

class ComplexityAnalyzer {
    constructor() {
        this.complexityKeywords = {
            high: {
                system: ['system', 'architecture', 'workflow', 'integration', 'complete', 'full', 'entire'],
                services: ['stripe', 'paypal', 'aws', 'redis', 'supabase', 'gemini', 'google', 'microsoft'],
                components: ['multiple', 'several', 'various', 'complex', 'advanced'],
                patterns: ['microservices', 'cqrs', 'event-sourcing', 'distributed'],
                scope: ['platform', 'ecosystem', 'suite', 'framework']
            },
            medium: {
                features: ['endpoint', 'handler', 'service', 'component', 'module', 'api'],
                operations: ['database', 'storage', 'validation', 'processing', 'conversion'],
                integration: ['integration', 'connection', 'interface', 'adapter'],
                size: ['application', 'manager', 'controller', 'processor']
            },
            low: {
                fixes: ['fix', 'bug', 'error', 'issue', 'problem'],
                changes: ['update', 'modify', 'change', 'adjust', 'refactor'],
                additions: ['add', 'create', 'implement', 'build', 'small'],
                maintenance: ['cleanup', 'optimize', 'improve', 'enhance']
            }
        };

        this.backendIndicators = ['api', 'endpoint', 'queue', 'worker', 'database', 'redis', 'service', 'integration'];
        this.frontendIndicators = ['ui', 'component', 'page', 'interface', 'frontend', 'dashboard'];
    }

    /**
     * Analyze feature description and return complexity assessment
     */
    analyze(description) {
        const analysis = {
            description,
            complexity: 'medium',
            score: 5,
            type: 'backend',
            components: [],
            externalServices: [],
            estimatedTasks: 2,
            estimatedTime: '30-45 minutes',
            workflow: 'standard',
            risks: [],
            suggestions: []
        };

        // Convert to lowercase for analysis
        const desc = description.toLowerCase();

        // 1. Keyword scoring
        const keywordScore = this.calculateKeywordScore(desc);

        // 2. Component analysis
        const componentAnalysis = this.analyzeComponents(desc);

        // 3. External service detection
        const serviceAnalysis = this.detectExternalServices(desc);

        // 4. Pattern detection
        const patternAnalysis = this.detectPatterns(desc);

        // 5. Calculate final score
        analysis.score = this.calculateFinalScore(keywordScore, componentAnalysis, serviceAnalysis, patternAnalysis);

        // 6. Determine complexity and workflow
        this.determineComplexity(analysis);

        // 7. Set type-specific attributes
        this.setTypeSpecificAttributes(analysis, desc);

        // 8. Generate risks and suggestions
        this.generateRisksAndSuggestions(analysis, componentAnalysis, serviceAnalysis);

        return analysis;
    }

    /**
     * Calculate score based on keyword analysis
     */
    calculateKeywordScore(description) {
        let score = 5; // Base score (medium)

        // High complexity keywords
        for (const keyword of this.complexityKeywords.high.system) {
            if (description.includes(keyword)) score += 2;
        }
        for (const keyword of this.complexityKeywords.high.services) {
            if (description.includes(keyword)) score += 2;
        }
        for (const keyword of this.complexityKeywords.high.components) {
            if (description.includes(keyword)) score += 1;
        }

        // Medium complexity keywords
        for (const keyword of this.complexityKeywords.medium.features) {
            if (description.includes(keyword)) score += 1;
        }

        // Low complexity keywords (reduce score)
        for (const keyword of this.complexityKeywords.low.fixes) {
            if (description.includes(keyword)) score -= 1;
        }
        for (const keyword of this.complexityKeywords.low.changes) {
            if (description.includes(keyword)) score -= 1;
        }

        return Math.max(1, Math.min(10, score));
    }

    /**
     * Analyze components mentioned in description
     */
    analyzeComponents(description) {
        const components = [];
        const componentPatterns = [
            { pattern: /\bapi\b|\bendpoint\b|\bservice\b/g, type: 'api' },
            { pattern: /\bqueue\b|\bworker\b|\bjob\b/g, type: 'queue' },
            { pattern: /\bdatabase\b|\bdb\b|\bsql\b/g, type: 'database' },
            { pattern: /\bui\b|\bcomponent\b|\bpage\b/g, type: 'ui' },
            { pattern: /\bauth\b|\bauthentication\b|\blogin\b/g, type: 'auth' },
            { pattern: /\bpayment\b|\bbilling\b|\bstripe\b/g, type: 'payment' },
            { pattern: /\bnotification\b|\bemail\b|\bsms\b/g, type: 'notification' },
            { pattern: /\bsearch\b|\belasticsearch\b|\bindex\b/g, type: 'search' },
            { pattern: /\bfile\b|\bupload\b|\bstorage\b/g, type: 'file' },
            { pattern: /\bcache\b|\brediscache\b/g, type: 'cache' }
        ];

        for (const { pattern, type } of componentPatterns) {
            const matches = description.match(pattern);
            if (matches) {
                components.push({
                    type,
                    count: matches.length,
                    keywords: [...new Set(matches)]
                });
            }
        }

        return components;
    }

    /**
     * Detect external services mentioned
     */
    detectExternalServices(description) {
        const services = [];
        const servicePatterns = [
            { pattern: /\bstripe\b/g, service: 'Stripe', type: 'payment' },
            { pattern: /\bredis\b/g, service: 'Redis', type: 'cache' },
            { pattern: /\bsupabase\b/g, service: 'Supabase', type: 'database' },
            { pattern: /\bgemini\b/g, service: 'Google Gemini', type: 'ai' },
            { pattern: /\baws\b|\bamazon\b/g, service: 'AWS', type: 'cloud' },
            { pattern: /\bgoogle\b|\bgcp\b/g, service: 'Google Cloud', type: 'cloud' },
            { pattern: /\bpaypal\b/g, service: 'PayPal', type: 'payment' },
            { pattern: /\bsendgrid\b/g, service: 'SendGrid', type: 'email' },
            { pattern: /\bmailgun\b/g, service: 'Mailgun', type: 'email' },
            { pattern: /\bpusher\b/g, service: 'Pusher', type: 'realtime' }
        ];

        for (const { pattern, service, type } of servicePatterns) {
            if (pattern.test(description)) {
                services.push({ service, type });
            }
        }

        return services;
    }

    /**
     * Detect architectural patterns
     */
    detectPatterns(description) {
        const patterns = [];
        const patternKeywords = {
            microservices: ['microservice', 'micro-services'],
            cqrs: ['cqrs', 'command query'],
            event_sourcing: ['event sourcing', 'event-store'],
            distributed: ['distributed', 'cluster', 'scalable'],
            real_time: ['real-time', 'websocket', 'streaming'],
            async: ['async', 'queue', 'worker', 'background']
        };

        for (const [pattern, keywords] of Object.entries(patternKeywords)) {
            for (const keyword of keywords) {
                if (description.includes(keyword)) {
                    patterns.push(pattern);
                    break;
                }
            }
        }

        return patterns;
    }

    /**
     * Calculate final complexity score
     */
    calculateFinalScore(keywordScore, componentAnalysis, serviceAnalysis, patternAnalysis) {
        let finalScore = keywordScore;

        // Component impact
        finalScore += componentAnalysis.components.length * 0.5;

        // External service impact
        finalScore += serviceAnalysis.services.length * 1;

        // Pattern impact
        finalScore += patternAnalysis.patterns.length * 0.8;

        return Math.round(finalScore);
    }

    /**
     * Determine complexity category and workflow
     */
    determineComplexity(analysis) {
        if (analysis.score <= 3) {
            analysis.complexity = 'low';
            analysis.workflow = 'quick';
            analysis.estimatedTasks = 1;
            analysis.estimatedTime = '5-15 minutes';
        } else if (analysis.score <= 7) {
            analysis.complexity = 'medium';
            analysis.workflow = 'standard';
            analysis.estimatedTasks = Math.ceil(analysis.score / 2);
            analysis.estimatedTime = '20-45 minutes';
        } else {
            analysis.complexity = 'high';
            analysis.workflow = 'comprehensive';
            analysis.estimatedTasks = Math.ceil(analysis.score / 1.5);
            analysis.estimatedTime = '1-3 hours';
        }

        // Cap maximum tasks
        analysis.estimatedTasks = Math.min(8, Math.max(1, analysis.estimatedTasks));
    }

    /**
     * Set type-specific attributes
     */
    setTypeSpecificAttributes(analysis, description) {
        // Backend vs Frontend detection
        const backendScore = this.backendIndicators.filter(ind => description.includes(ind)).length;
        const frontendScore = this.frontendIndicators.filter(ind => description.includes(ind)).length;

        analysis.type = backendScore >= frontendScore ? 'backend' : 'frontend';

        // Add type-specific suggestions
        if (analysis.type === 'backend') {
            analysis.suggestions.push('Test with real external services using /test-env');
            analysis.suggestions.push('Implement API endpoints first for immediate testing');
        } else {
            analysis.suggestions.push('Create responsive design variations');
            analysis.suggestions.push('Test with multiple screen sizes');
        }
    }

    /**
     * Generate risks and suggestions
     */
    generateRisksAndSuggestions(analysis, componentAnalysis, serviceAnalysis) {
        // Component-based risks
        for (const component of componentAnalysis.components) {
            switch (component.type) {
                case 'api':
                    analysis.risks.push('API versioning and backward compatibility');
                    analysis.suggestions.push('Implement API versioning from start');
                    break;
                case 'queue':
                    analysis.risks.push('Message ordering and dead letter queues');
                    analysis.suggestions.push('Plan for queue monitoring and error handling');
                    break;
                case 'database':
                    analysis.risks.push('Schema migrations and data integrity');
                    analysis.suggestions.push('Design backward-compatible migrations');
                    break;
                case 'payment':
                    analysis.risks.push('Payment security and compliance');
                    analysis.suggestions.push('Follow PCI compliance guidelines');
                    break;
            }
        }

        // External service risks
        for (const service of serviceAnalysis.services) {
            analysis.risks.push(`${service.service} service dependency and rate limiting`);
            analysis.suggestions.push(`Implement retry logic and circuit breaker for ${service.service}`);
        }

        // Deduplicate suggestions
        analysis.suggestions = [...new Set(analysis.suggestions)];

        // Add general suggestions based on complexity
        if (analysis.complexity === 'high') {
            analysis.suggestions.push('Consider breaking into multiple milestones');
            analysis.suggestions.push('Plan for comprehensive logging and monitoring');
        } else if (analysis.complexity === 'low') {
            analysis.suggestions.push('Focus on core functionality only');
            analysis.suggestions.push('Add comprehensive tests for edge cases');
        }
    }

    /**
     * Generate workflow recommendations
     */
    generateWorkflowRecommendation(analysis) {
        const recommendation = {
            workflow: analysis.workflow,
            estimatedTime: analysis.estimatedTime,
            tasks: analysis.estimatedTasks,
            command: this.generateCommand(analysis),
            steps: this.generateSteps(analysis)
        };

        return recommendation;
    }

    /**
     * Generate appropriate command based on analysis
     */
    generateCommand(analysis) {
        switch (analysis.workflow) {
            case 'quick':
                return `/task "${analysis.description}" --quick`;
            case 'standard':
                return `/task "${analysis.description}"`;
            case 'comprehensive':
                return `/task "${analysis.description}" --deep`;
            default:
                return `/task "${analysis.description}"`;
        }
    }

    /**
     * Generate implementation steps
     */
    generateSteps(analysis) {
        const baseSteps = [
            'Environment validation with /test-env',
            'Write tests first (TDD - Red phase)',
            'Implement minimum functionality (Green phase)',
            'Refactor and optimize (Refactor phase)',
            'Run 100% validation (build, lint, test)'
        ];

        if (analysis.complexity === 'high') {
            return [
                'Create comprehensive context with /fcs',
                'Break into milestones with /task --milestone',
                ...baseSteps.map(step => `Execute for each milestone: ${step}`),
                'Full integration testing',
                'Performance and security validation'
            ];
        } else if (analysis.complexity === 'low') {
            return baseSteps.slice(1, 4); // Skip context and integration testing
        }

        return baseSteps;
    }
}

module.exports = ComplexityAnalyzer;