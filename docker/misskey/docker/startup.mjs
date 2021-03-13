import * as fs from 'fs';
import * as child_process from 'child_process';

const config_text =
     `url: https://social.namachan10777.dev`
  + `\nport: 3000`
  + `\ndb:`
  + `\n  host: db`
  + `\n  port: 5432`
  + `\n  db: ${process.env.POSTGRES_DB}`
  + `\n  user: ${process.env.POSTGRES_USER}`
  + `\n  pass: ${process.env.POSTGRES_PASSWORD}`
  + `\nredis:`
  + `\n  host: redis`
  + `\n  port: 6379`
  + `\nelasticsearch:`
  + `\n  host: es`
  + `\n  port: 9200`
  + `\n  ssl: false`
  + `\n  user: elastic`
  + `\n  pass: ${process.env.ELASTICSEARCH_PASSWORD}`
  + `\nid: aid`;

const decoder = new TextDecoder('utf-8');
fs.writeFileSync('/misskey/.config/default.yml', config_text);
const misskey = child_process.spawn('npm', ['run', 'migrateandstart'], { cwd: '/misskey' });
misskey.stdout.on('data', (data) => { process.stdout.write(decoder.decode(new Uint8Array(data))); })
misskey.stderr.on('data', (data) => { process.stderr.write(decoder.decode(new Uint8Array(data))); });
misskey.on('error', (error) => { console.log(error) });
