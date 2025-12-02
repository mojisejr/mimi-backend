/**
 * Task Generation Engine for Smart Planning
 * Generates atomic tasks based on context analysis and complexity
 */

class TaskGenerator {
    constructor() {
        this.taskPatterns = {
            api: [
                'Create {entity} model and validation',
                'Implement {operation} {entity} endpoint',
                'Add {entity} error handling and tests'
            ],
            database: [
                'Create {entity} table migration',
                'Add database indexes for {entity}',
                'Implement {entity} repository layer'
            ],
            auth: [
                'Implement user authentication middleware',
                'Create JWT token service',
                'Add authorization for protected routes'
            ],
            queue: [
                'Set up Redis queue connection',
                'Create job producer for {entity}',
                'Implement background worker for {entity} processing'
            ],
            external: [
                'Integrate {service} API client',
                'Add {service} error handling',
                'Create {service} service layer'
            ],
            ui: [
                'Create {component} React component',
                'Add {component} form validation',
                'Implement {component} state management'
            ]
        };

        this.milestonePatterns = {
            week1: 'Core foundation',
            week2: 'Essential features',
            week3: 'Advanced features',
            week4: 'Polish and optimization'
        };
    }

    /**
     * Generate tasks based on context analysis
     */
    generateTasks(contextAnalysis, options = {}) {
        const {
            maxTasks = 8,
            complexity = 'medium',
            type = 'backend'
        } = options;

        const tasks = [];

        // Extract entities and features from context
        const entities = this.extractEntities(contextAnalysis);
        const features = this.extractFeatures(contextAnalysis);
        const patterns = this.detectImplementationPatterns(contextAnalysis);

        // Generate tasks based on detected patterns
        for (const pattern of patterns) {
            const patternTasks = this.generateTasksForPattern(pattern, entities, features);
            tasks.push(...patternTasks);

            // Respect max task limit
            if (tasks.length >= maxTasks) break;
        }

        // Add cross-cutting concerns
        const crosscuttingTasks = this.generateCrosscuttingTasks(type, complexity);
        tasks.push(...crosscuttingTasks);

        // Sort and number tasks
        return this.formatTasks(tasks.slice(0, maxTasks));
    }

    /**
     * Extract entities from context analysis
     */
    extractEntities(analysis) {
        const entities = new Set();

        // Common entity patterns
        const entityPatterns = [
            /user/gi, /customer/gi, /product/gi, /order/gi, /payment/gi,
            /booking/gi, /reservation/gi, /item/gi, /resource/gi, /record/gi,
            /profile/gi, /account/gi, /session/gi, /token/gi, /message/gi
        ];

        if (analysis.description) {
            for (const pattern of entityPatterns) {
                const matches = analysis.description.match(pattern);
                if (matches) {
                    matches.forEach(match => entities.add(match.toLowerCase()));
                }
            }
        }

        // Extract from components
        if (analysis.components) {
            for (const component of analysis.components) {
                if (component.entities) {
                    component.entities.forEach(entity => entities.add(entity));
                }
            }
        }

        return Array.from(entities);
    }

    /**
     * Extract features from context analysis
     */
    extractFeatures(analysis) {
        const features = [];

        const featurePatterns = [
            { pattern: /authentication|auth|login/gi, feature: 'authentication' },
            { pattern: /payment|billing|charge/gi, feature: 'payment' },
            { pattern: /notification|email|sms/gi, feature: 'notification' },
            { pattern: /search|filter|query/gi, feature: 'search' },
            { pattern: /upload|file|image/gi, feature: 'file_upload' },
            { pattern: /queue|worker|background/gi, feature: 'queue_processing' },
            { pattern: /api|endpoint|service/gi, feature: 'api_layer' },
            { pattern: /database|storage|persistence/gi, feature: 'data_layer' },
            { pattern: /ui|interface|frontend/gi, feature: 'user_interface' }
        ];

        if (analysis.description) {
            for (const { pattern, feature } of featurePatterns) {
                if (pattern.test(analysis.description)) {
                    features.push(feature);
                }
            }
        }

        return [...new Set(features)];
    }

    /**
     * Detect implementation patterns needed
     */
    detectImplementationPatterns(analysis) {
        const patterns = [];

        // Check for API requirements
        if (analysis.description.includes('api') ||
            analysis.description.includes('endpoint') ||
            analysis.features.includes('api_layer')) {
            patterns.push('api');
        }

        // Check for database requirements
        if (analysis.description.includes('database') ||
            analysis.description.includes('storage') ||
            analysis.features.includes('data_layer')) {
            patterns.push('database');
        }

        // Check for auth requirements
        if (analysis.description.includes('auth') ||
            analysis.description.includes('login') ||
            analysis.features.includes('authentication')) {
            patterns.push('auth');
        }

        // Check for queue requirements
        if (analysis.description.includes('queue') ||
            analysis.description.includes('worker') ||
            analysis.features.includes('queue_processing')) {
            patterns.push('queue');
        }

        // Check for external service requirements
        if (analysis.externalServices && analysis.externalServices.length > 0) {
            patterns.push('external');
        }

        // Check for UI requirements
        if (analysis.type === 'frontend' ||
            analysis.description.includes('ui') ||
            analysis.features.includes('user_interface')) {
            patterns.push('ui');
        }

        // Default to api if no specific patterns found
        if (patterns.length === 0) {
            patterns.push('api');
        }

        return patterns;
    }

    /**
     * Generate tasks for a specific pattern
     */
    generateTasksForPattern(pattern, entities, features) {
        const tasks = [];
        const patternTasks = this.taskPatterns[pattern] || this.taskPatterns.api;

        for (const template of patternTasks) {
            let task = template;

            // Replace entity placeholders
            if (entities.length > 0 && task.includes('{entity}')) {
                task = task.replace('{entity}', entities[0]);
            }

            // Replace operation placeholders
            if (task.includes('{operation}')) {
                const operations = ['Create', 'Read', 'Update', 'Delete'];
                const operation = operations[Math.floor(Math.random() * operations.length)];
                task = task.replace('{operation}', operation.toLowerCase());
            }

            // Replace service placeholders
            if (task.includes('{service}')) {
                if (features.length > 0) {
                    task = task.replace('{service}', features[0]);
                } else {
                    task = task.replace('{service}', 'external');
                }
            }

            // Replace component placeholders
            if (task.includes('{component}')) {
                if (entities.length > 0) {
                    task = task.replace('{component}', entities[0]);
                } else {
                    task = task.replace('{component}', 'Component');
                }
            }

            tasks.push({
                title: task,
                pattern,
                priority: this.calculateTaskPriority(pattern, task)
            });
        }

        return tasks;
    }

    /**
     * Generate cross-cutting tasks
     */
    generateCrosscuttingTasks(type, complexity) {
        const tasks = [];

        // Testing tasks
        tasks.push({
            title: 'Create comprehensive test suite for all features',
            pattern: 'testing',
            priority: 'high'
        });

        // Documentation tasks
        if (complexity === 'high') {
            tasks.push({
                title: 'Create API documentation and usage examples',
                pattern: 'documentation',
                priority: 'medium'
            });
        }

        // Performance tasks
        if (type === 'backend' && complexity === 'high') {
            tasks.push({
                title: 'Add performance monitoring and optimization',
                pattern: 'performance',
                priority: 'medium'
            });
        }

        // Security tasks
        if (type === 'backend') {
            tasks.push({
                title: 'Implement security best practices and validation',
                pattern: 'security',
                priority: 'high'
            });
        }

        return tasks;
    }

    /**
     * Calculate task priority
     */
    calculateTaskPriority(pattern, task) {
        // High priority tasks
        const highPriorityKeywords = ['create', 'implement', 'set up', 'add'];
        const lowPriorityKeywords = ['documentation', 'optimization', 'polish'];

        if (highPriorityKeywords.some(keyword => task.toLowerCase().includes(keyword))) {
            return 'high';
        }

        if (lowPriorityKeywords.some(keyword => task.toLowerCase().includes(keyword))) {
            return 'low';
        }

        return 'medium';
    }

    /**
     * Format and number tasks
     */
    formatTasks(tasks) {
        // Sort by priority
        tasks.sort((a, b) => {
            const priorityOrder = { high: 3, medium: 2, low: 1 };
            return priorityOrder[b.priority] - priorityOrder[a.priority];
        });

        // Add task numbers and format
        return tasks.map((task, index) => ({
            id: index + 1,
            title: task.title,
            pattern: task.pattern,
            priority: task.priority,
            estimatedTime: this.estimateTaskTime(task.pattern, task.priority),
            dependencies: this.identifyDependencies(task, tasks.slice(0, index))
        }));
    }

    /**
     * Estimate task time
     */
    estimateTaskTime(pattern, priority) {
        const baseTimes = {
            api: 30,
            database: 45,
            auth: 60,
            queue: 50,
            external: 40,
            ui: 35,
            testing: 25,
            documentation: 20,
            performance: 35,
            security: 40
        };

        const baseTime = baseTimes[pattern] || 30;
        const priorityMultiplier = {
            high: 1.2,
            medium: 1.0,
            low: 0.8
        };

        return Math.round(baseTime * priorityMultiplier[priority]);
    }

    /**
     * Identify task dependencies
     */
    identifyDependencies(currentTask, previousTasks) {
        const dependencies = [];

        // Common dependency rules
        if (currentTask.pattern === 'external') {
            const apiTask = previousTasks.find(t => t.pattern === 'api');
            if (apiTask) {
                dependencies.push(`API layer (${apiTask.title})`);
            }
        }

        if (currentTask.pattern === 'queue') {
            const dbTask = previousTasks.find(t => t.pattern === 'database');
            if (dbTask) {
                dependencies.push(`Database schema (${dbTask.title})`);
            }
        }

        if (currentTask.pattern === 'testing') {
            const implementationTasks = previousTasks.filter(t =>
                ['api', 'database', 'auth', 'external'].includes(t.pattern)
            );
            if (implementationTasks.length > 0) {
                dependencies.push(`Implementation tasks (${implementationTasks.length} tasks)`);
            }
        }

        return dependencies;
    }

    /**
     * Generate milestone breakdown
     */
    generateMilestones(tasks, weeks = 4) {
        const tasksPerWeek = Math.ceil(tasks.length / weeks);
        const milestones = [];

        for (let week = 0; week < weeks; week++) {
            const startIndex = week * tasksPerWeek;
            const endIndex = Math.min(startIndex + tasksPerWeek, tasks.length);
            const weekTasks = tasks.slice(startIndex, endIndex);

            milestones.push({
                week: week + 1,
                title: `Week ${week + 1}: ${this.milestonePatterns[`week${week + 1}`] || 'Implementation phase'}`,
                tasks: weekTasks,
                estimatedTime: weekTasks.reduce((sum, task) => sum + task.estimatedTime, 0),
                deliverables: this.generateMilestoneDeliverables(weekTasks)
            });
        }

        return milestones;
    }

    /**
     * Generate milestone deliverables
     */
    generateMilestoneDeliverables(tasks) {
        const deliverables = [];

        for (const task of tasks) {
            switch (task.pattern) {
                case 'api':
                    deliverables.push(`${task.title} (API endpoints)`);
                    break;
                case 'database':
                    deliverables.push(`${task.title} (Database schema)`);
                    break;
                case 'auth':
                    deliverables.push(`${task.title} (Security features)`);
                    break;
                case 'queue':
                    deliverables.push(`${task.title} (Background processing)`);
                    break;
                case 'external':
                    deliverables.push(`${task.title} (External integration)`);
                    break;
                case 'ui':
                    deliverables.push(`${task.title} (User interface)`);
                    break;
                default:
                    deliverables.push(task.title);
            }
        }

        return deliverables;
    }

    /**
     * Generate implementation plan
     */
    generateImplementationPlan(analysis, options = {}) {
        const tasks = this.generateTasks(analysis, options);
        const plan = {
            totalTasks: tasks.length,
            estimatedTotalTime: tasks.reduce((sum, task) => sum + task.estimatedTime, 0),
            tasks,
            milestones: options.milestones ? this.generateMilestones(tasks, options.weeks || 4) : null,
            risks: this.generateImplementationRisks(tasks, analysis),
            successCriteria: this.generateSuccessCriteria(tasks, analysis),
            nextSteps: this.generateNextSteps(tasks)
        };

        return plan;
    }

    /**
     * Generate implementation risks
     */
    generateImplementationRisks(tasks, analysis) {
        const risks = [];

        // External service risks
        if (analysis.externalServices && analysis.externalServices.length > 0) {
            risks.push({
                type: 'external_dependency',
                description: `Reliance on external services: ${analysis.externalServices.map(s => s.service).join(', ')}`,
                mitigation: 'Implement retry logic, circuit breakers, and fallback strategies'
            });
        }

        // Complex task risks
        const complexTasks = tasks.filter(task => task.estimatedTime > 60);
        if (complexTasks.length > 0) {
            risks.push({
                type: 'complexity',
                description: `${complexTasks.length} tasks require significant implementation time`,
                mitigation: 'Break down complex tasks further, allocate additional time'
            });
        }

        // Dependency risks
        const tasksWithDeps = tasks.filter(task => task.dependencies.length > 0);
        if (tasksWithDeps.length > tasks.length * 0.5) {
            risks.push({
                type: 'dependencies',
                description: 'High dependency chain may cause delays',
                mitigation: 'Identify critical path, consider parallel execution where possible'
            });
        }

        return risks;
    }

    /**
     * Generate success criteria
     */
    generateSuccessCriteria(tasks, analysis) {
        const criteria = [
            'All generated tasks completed successfully',
            '100% test coverage for new functionality',
            'All quality gates passed (build, lint, test)',
            'External services integrated and tested',
            'Performance benchmarks met',
            'Security validation completed'
        ];

        // Add feature-specific criteria
        if (analysis.features.includes('authentication')) {
            criteria.push('User authentication flow working end-to-end');
        }

        if (analysis.features.includes('payment')) {
            criteria.push('Payment processing tested with sandbox environment');
        }

        return criteria;
    }

    /**
     * Generate next steps
     */
    generateNextSteps(tasks) {
        const steps = [
            'Review and approve generated task list',
            'Verify external service access with `/test-env`',
            'Set up development environment',
            'Begin implementation with highest priority tasks',
            'Track progress and update estimates as needed'
        ];

        if (tasks.length > 0) {
            steps.unshift(`Start with: ${tasks[0].title} (estimated ${tasks[0].estimatedTime} minutes)`);
        }

        return steps;
    }
}

module.exports = TaskGenerator;