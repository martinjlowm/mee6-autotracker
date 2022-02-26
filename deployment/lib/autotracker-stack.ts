import {
  Duration,
  Stack,
  StackProps,
  aws_secretsmanager as sm,
  aws_lambda as lambda,
  aws_dynamodb as dynamodb,
  aws_events as events,
  aws_events_targets as events_targets,
  aws_lambda_event_sources as lambda_event_sources,
  aws_apigateway as apigateway,
} from 'aws-cdk-lib';
import { RustFunction } from 'rust.aws-cdk-lambda';
import { Construct } from 'constructs';
import { LambdaIntegration } from 'aws-cdk-lib/aws-apigateway';
// import * as sqs from 'aws-cdk-lib/aws-sqs';

export class AutoTrackerStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const actionsTable = new dynamodb.Table(this, 'actions', {
      tableName: 'autotracker-actions',
      billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
      timeToLiveAttribute: 'ttl',
      stream: dynamodb.StreamViewType.OLD_IMAGE,
      partitionKey: {
        name: 'pk',
        type: dynamodb.AttributeType.STRING,
      },
      sortKey: {
        name: 'sk',
        type: dynamodb.AttributeType.STRING,
      },
    });

    const slackToken = new sm.Secret(this, 'slack-token');
    const harvestToken = new sm.Secret(this, 'harvest-token');

    const slackPrompt = new RustFunction(this, 'slack-prompt', {
      functionName: 'autotracker-slack-prompt',
      description: 'Prompt for hours to key in in Harvest',
      architecture: lambda.Architecture.ARM_64,
      memorySize: 128,
      timeout: Duration.seconds(10),
    });

    actionsTable.grantReadWriteData(slackPrompt);

    slackPrompt.addEnvironment('SLACK_TOKEN', slackToken.secretValue.toString());

    const mondayThroughFriday = '1-5';
    new events.Rule(this, 'trigger-schedule', {
      schedule: events.Schedule.cron({ hour: '9', minute: '0', weekDay: mondayThroughFriday }),
      targets: [new events_targets.LambdaFunction(slackPrompt)],
    });

    const adjustHours = new RustFunction(this, 'adjust-hours', {
      functionName: 'autotracker-adjust-hours',
      description: 'Adjust hours through webhook as clicked from Slack',
      architecture: lambda.Architecture.ARM_64,
      memorySize: 128,
      timeout: Duration.seconds(10),
    });

    actionsTable.grantReadWriteData(adjustHours);

    const api = new apigateway.RestApi(this, 'gateway');

    const autoTrackerResource = api.root.addResource('auto-tracker');
    const adjustHoursResource = autoTrackerResource.addResource('adjust-hours');
    adjustHoursResource.addMethod('POST', new LambdaIntegration(adjustHours), { apiKeyRequired: true });

    api.addApiKey('slack-webhook-key');

    const registerHours = new RustFunction(this, 'register-hours', {
      functionName: 'autotracker-register-hours',
      description: 'Register hours',
      architecture: lambda.Architecture.ARM_64,
      memorySize: 128,
      timeout: Duration.seconds(10),
    });
    registerHours.addEnvironment('HARVEST_TOKEN', harvestToken.secretValue.toString());
    registerHours.addEnvironment('HARVEST_ACCOUNT_ID', '203529');

    registerHours.addEventSource(
      new lambda_event_sources.DynamoEventSource(actionsTable, {
        startingPosition: lambda.StartingPosition.TRIM_HORIZON,
      }),
    );
    actionsTable.grantStreamRead(registerHours);
  }
}
