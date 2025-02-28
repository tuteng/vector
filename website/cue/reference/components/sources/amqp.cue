package metadata

components: sources: amqp: {
	title: "AMQP"

	features: {
		acknowledgements: false
		collect: {
			checkpoint: enabled: false
			from: {
				service: services.amqp
				interface: {
					socket: {
						api: {
							title: "AMQP protocol"
							url:   urls.amqp_protocol
						}
						direction: "incoming"
						port:      5672
						protocols: ["tcp"]
						ssl: "optional"
					}
				}
			}
		}
		multiline: enabled: false
	}

	classes: {
		commonly_used: true
		deployment_roles: ["aggregator"]
		delivery:      "at_least_once"
		development:   "beta"
		egress_method: "stream"
		stateful:      false
	}

	support: components._amqp.support

	installation: {
		platform_name: null
	}

	configuration: {
		connection: {
			description: "Connection options for the AMQP source."
			required:    true
			warnings: []
			type: object: {
				examples: []
				options: {
					connection_string: components._amqp.configuration.connection_string
				}
			}
		}
		group_id: {
			description: "The consumer group name to be used to consume events from AMQP."
			required:    true
			warnings: []
			type: string: {
				examples: ["consumer-group-name"]
				syntax: "literal"
			}
		}
		routing_key_field: {
			common:      true
			description: "The log field name to use for the AMQP routing key."
			required:    false
			warnings: []
			type: string: {
				default: "routing"
				examples: ["routing"]
				syntax: "literal"
			}
		}
		exchange_key: {
			common:      true
			description: "The log field name to use for the AMQP exchange key."
			required:    false
			warnings: []
			type: string: {
				default: "exchange"
				examples: ["exchange"]
				syntax: "literal"
			}
		}
		offset_key: {
			common:      true
			description: "The log field name to use for the AMQP offset key."
			required:    false
			warnings: []
			type: string: {
				default: "offset"
				examples: ["offset"]
				syntax: "literal"
			}
		}
	}

	output: logs: record: {
		description: "An individual AMQP record."
		fields: {
			message: {
				description: "The raw line from the AMQP record."
				required:    true
				type: string: {
					examples: ["53.126.150.246 - - [01/Oct/2020:11:25:58 -0400] \"GET /disintermediate HTTP/2.0\" 401 20308"]
					syntax: "literal"
				}
			}
			offset: {
				description: "The AMQP offset at the time the record was retrieved."
				required:    true
				type: uint: {
					examples: [100]
					unit: null
				}
			}
			timestamp: fields._current_timestamp & {
				description: "The timestamp encoded in the AMQP message or the current time if it cannot be fetched."
			}
			exchange: {
				description: "The AMQP exchange that the record came from."
				required:    true
				type: string: {
					examples: ["topic"]
					syntax: "literal"
				}
			}
		}
	}

	telemetry: metrics: {
		events_in_total:                      components.sources.internal_metrics.output.metrics.events_in_total
		consumer_offset_updates_failed_total: components.sources.internal_metrics.output.metrics.consumer_offset_updates_failed_total
		events_failed_total:                  components.sources.internal_metrics.output.metrics.events_failed_total
		processed_bytes_total:                components.sources.internal_metrics.output.metrics.processed_bytes_total
		processed_events_total:               components.sources.internal_metrics.output.metrics.processed_events_total
	}

	how_it_works: components._amqp.how_it_works
}
