#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { AutoTrackerStack } from '../lib/autotracker-stack';
export { AutoTrackerStack };

export default () => {
  const app = new cdk.App();

  new AutoTrackerStack(app, 'AutoTrackerStack', {});
};
