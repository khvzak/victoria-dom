#!/usr/bin/env perl

use Mojo::Base -strict;
use Mojo::UserAgent;
use Cwd qw(abs_path);
use File::Basename qw(dirname);
use File::Spec qw();
use Path::Tiny qw(path);

my @data;

# Extract named character references from HTML Living Standard
my $res = Mojo::UserAgent->new->get('https://html.spec.whatwg.org')->result;
for my $row ($res->dom('#named-character-references-table tbody > tr')->each) {
    my $entity     = $row->at('td > code')->text;
    my $codepoints = $row->children('td')->[1]->text;

    if ($codepoints =~ /^\s*U\+(\S+)(?:\s+U\+(\S+))?/) {
        push @data, [$entity, defined($2) ? "\\u{$1}\\u{$2}" : "\\u{$1}"];
    }
}

my $util_rs_file = File::Spec->catfile(dirname(abs_path($0)), '..', 'src', 'util.rs');

my $util_rs_data = path($util_rs_file)->slurp_utf8;

my $entities = join(",\n", map { '        "'.$_->[0].'" => "'.$_->[1].'"' } @data);
$util_rs_data =~ s/(?<=    static ref ENTITIES: HashMap<&'static str, &'static str> = hashmap!\[\n).+(?=    \];)/$entities,\n/s;

path($util_rs_file)->spew_utf8($util_rs_data);
