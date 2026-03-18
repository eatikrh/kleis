#!/usr/bin/env perl
# Check all markdown links in the repository

use strict;
use warnings;
use File::Find;
use File::Spec;
use Cwd 'abs_path';

my $repo_root = "/Users/eatik_1/Documents/git/cee/kleis";
my %exclude_dirs = map { $_ => 1 } qw(node_modules target .git vendor);

my @md_files;
my $broken_count = 0;
my $total_links = 0;

# Find all .md files
find(sub {
    return if $File::Find::dir =~ m{/(node_modules|target|\.git|vendor)(/|$)};
    push @md_files, $File::Find::name if /\.md$/;
}, $repo_root);

@md_files = sort @md_files;

print "🔍 Checking all markdown links in repository...\n\n";
print "Found " . scalar(@md_files) . " markdown files\n\n";

my @broken_links;

for my $md_file (@md_files) {
    open my $fh, '<:utf8', $md_file or do {
        warn "Cannot read $md_file: $!\n";
        next;
    };
    
    my $content = do { local $/; <$fh> };
    close $fh;
    
    my $md_dir = $md_file;
    $md_dir =~ s{/[^/]+$}{};
    
    # Extract markdown links: [text](link)
    while ($content =~ /\[([^\]]+)\]\(([^)]+)\)/g) {
        my ($text, $link) = ($1, $2);
        
        # Skip external links and anchors
        next if $link =~ m{^(https?://|mailto:|#)};
        
        # Remove anchor from link
        $link =~ s/#.*$//;
        next unless $link;  # Skip empty links
        
        $total_links++;
        
        # Resolve path
        my $target;
        if ($link =~ m{^/}) {
            # Absolute from repo root
            $target = "$repo_root$link";
        } else {
            # Relative from markdown file
            $target = File::Spec->catfile($md_dir, $link);
        }
        
        # Normalize and check existence
        $target = abs_path($target) if -e $target;
        
        unless (-e $target || -d $target) {
            my $rel_md = $md_file;
            $rel_md =~ s{^\Q$repo_root\E/}{};
            
            push @broken_links, {
                file => $rel_md,
                text => $text,
                link => $link,
                target => $target
            };
        }
    }
}

# Report results
if (@broken_links) {
    print "❌ Found " . scalar(@broken_links) . " broken links:\n\n";
    
    my $current_file = '';
    for my $broken (@broken_links) {
        if ($broken->{file} ne $current_file) {
            print "\n📄 $broken->{file}\n";
            $current_file = $broken->{file};
        }
        print "   ❌ [$broken->{text}]($broken->{link})\n";
        my $rel_target = $broken->{target};
        $rel_target =~ s{^\Q$repo_root\E/}{};
        print "      → Resolves to: $rel_target\n";
    }
}

print "\n📊 Summary:\n";
print "   Markdown files: " . scalar(@md_files) . "\n";
print "   Total links checked: $total_links\n";
print "   Broken links: " . scalar(@broken_links) . "\n\n";

if (@broken_links) {
    print "⚠️  Please fix broken links\n";
    exit 1;
} else {
    print "✅ All links are valid!\n";
    exit 0;
}

